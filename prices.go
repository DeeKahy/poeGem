package main

// Prices come from poe.ninja (its listing-based skill gem overview, and the
// currency exchange for divines and lab offerings), cached per league.

import (
	"context"
	"encoding/base64"
	"encoding/json"
	"fmt"
	"net/url"
	"strconv"
	"strings"
	"sync"
	"time"
)

const (
	ninjaBase = "https://poe.ninja/poe1/api/economy"
	priceTTL  = 30 * time.Minute
)

// Quote is [chaos, listings] for one gem at one level/quality.
type Quote [2]float64

type Prices struct {
	League  string                      `json:"league"`
	Fetched time.Time                   `json:"fetched"`
	Divine  float64                     `json:"divine"`
	Offers  map[string]float64          `json:"offerings"` // lab offerings, chaos
	Gems    map[string]map[string]Quote `json:"gems"`      // name -> variant -> quote
	// Icon-derived colours, used only for gems RePoE has no colour for yet
	// (a brand-new base gem on patch day).
	IconColour map[string]string `json:"-"`
}

type priceCache struct {
	mu      sync.Mutex
	leagues map[string]*Prices
	locks   map[string]*sync.Mutex
}

func (pc *priceCache) lock(league string) *sync.Mutex {
	pc.mu.Lock()
	defer pc.mu.Unlock()
	if pc.locks[league] == nil {
		pc.locks[league] = &sync.Mutex{}
	}
	return pc.locks[league]
}

// variant names the uncorrupted level/quality steps poe.ninja prices.
// Corrupted gems cannot go into the Font, so they are left out.
func variant(level, quality int) string {
	if quality > 0 {
		return fmt.Sprintf("%d/%d", level, quality)
	}
	return strconv.Itoa(level)
}

func (s *Server) prices(ctx context.Context, league string) (*Prices, error) {
	l := s.pc.lock(league)
	l.Lock()
	defer l.Unlock()
	s.pc.mu.Lock()
	p := s.pc.leagues[league]
	s.pc.mu.Unlock()
	if p != nil && time.Since(p.Fetched) < priceTTL {
		return p, nil
	}
	np, err := s.fetchPrices(ctx, league)
	if err != nil {
		if p != nil {
			return p, nil // stale beats nothing
		}
		return nil, err
	}
	s.pc.mu.Lock()
	s.pc.leagues[league] = np
	s.pc.mu.Unlock()
	return np, nil
}

func (s *Server) fetchPrices(ctx context.Context, league string) (*Prices, error) {
	q := url.Values{"league": {league}}
	p := &Prices{League: league, Fetched: time.Now(), Offers: map[string]float64{},
		Gems: map[string]map[string]Quote{}, IconColour: map[string]string{}}

	q.Set("type", "SkillGem")
	var sg struct {
		Lines []struct {
			Name         string  `json:"name"`
			Icon         string  `json:"icon"`
			GemLevel     int     `json:"gemLevel"`
			GemQuality   int     `json:"gemQuality"`
			Corrupted    bool    `json:"corrupted"`
			ChaosValue   float64 `json:"chaosValue"`
			ListingCount int     `json:"listingCount"`
		} `json:"lines"`
	}
	if err := fetchJSON(ctx, s.http, ninjaBase+"/stash/current/item/overview?"+q.Encode(), s.ua, &sg); err != nil {
		return nil, err
	}
	if len(sg.Lines) == 0 {
		return nil, fmt.Errorf("poe.ninja has no gem prices for %q", league)
	}
	for _, l := range sg.Lines {
		if l.Corrupted || l.GemLevel == 0 {
			continue
		}
		v := variant(l.GemLevel, l.GemQuality)
		if p.Gems[l.Name] == nil {
			p.Gems[l.Name] = map[string]Quote{}
		}
		p.Gems[l.Name][v] = Quote{l.ChaosValue, float64(l.ListingCount)}
		if c := iconColour(l.Icon); c != "" {
			p.IconColour[l.Name] = c
		}
	}

	for _, typ := range []string{"Currency", "Fragment"} {
		q.Set("type", typ)
		var ex struct {
			Items []struct{ ID, Name string } `json:"items"`
			Lines []struct {
				ID           string  `json:"id"`
				PrimaryValue float64 `json:"primaryValue"`
			} `json:"lines"`
		}
		if err := fetchJSON(ctx, s.http, ninjaBase+"/exchange/current/overview?"+q.Encode(), s.ua, &ex); err != nil {
			continue // optional extras
		}
		names := map[string]string{}
		for _, it := range ex.Items {
			names[it.ID] = it.Name
		}
		for _, l := range ex.Lines {
			n := names[l.ID]
			switch {
			case n == "Divine Orb":
				p.Divine = l.PrimaryValue
			case strings.HasSuffix(n, " to the Goddess"):
				p.Offers[n] = l.PrimaryValue
			}
		}
	}
	return p, nil
}

// iconColour reads the colour GGG bakes into a gem icon URL: the base64
// segment is JSON with a "gd" field (5/6 red, 9/10 green, 13/14 blue).
// The old poeGem used this as its only source; here it is a fallback.
func iconColour(icon string) string {
	_, rest, ok := strings.Cut(icon, "/image/")
	if !ok {
		return ""
	}
	seg, _, _ := strings.Cut(rest, "/")
	b, err := base64.RawStdEncoding.DecodeString(strings.TrimRight(seg, "="))
	if err != nil {
		return ""
	}
	var parts []json.RawMessage
	if json.Unmarshal(b, &parts) != nil || len(parts) < 3 {
		return ""
	}
	var o struct {
		GD int `json:"gd"`
	}
	json.Unmarshal(parts[2], &o)
	switch o.GD {
	case 5, 6:
		return "r"
	case 9, 10:
		return "g"
	case 13, 14:
		return "b"
	}
	return ""
}
