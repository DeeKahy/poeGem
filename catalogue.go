package main

// The catalogue answers "which gems can the Divine Font give, and what
// colour is each?" without any hand-kept list, so it follows the game from
// season to season:
//
//   - GGG's trade catalogue (/api/trade/data/items) lists every gem. A
//     transfigured gem carries a discriminator ("disc") and its "type" is the
//     base gem it comes from. That is GGG's own answer, unlike the old
//     `" of "` name test that counted Herald of Ash and Orb of Storms.
//   - RePoE gives the colour of each base gem; a transfigured gem has its
//     base gem's colour.
//
// It is rebuilt daily. When the set of Font outcomes changes, the change is
// recorded so the page can say what a patch added or removed.

import (
	"context"
	"encoding/json"
	"fmt"
	"net/http"
	"os"
	"path/filepath"
	"sort"
	"strings"
	"time"
)

const (
	tradeItemsURL = "https://www.pathofexile.com/api/trade/data/items"
	repoeGemsURL  = "https://repoe-fork.github.io/gems.min.json"
)

type Gem struct {
	Name   string `json:"name"`
	Base   string `json:"base"`   // the gem it transfigures from ("" for a base gem)
	Colour string `json:"colour"` // r, g, b, or "" if unknown
	// Font: a possible outcome of the Divine Font's transfigure options.
	Font bool `json:"font"`
	// Input: an active skill gem you could put into the Font.
	Input bool `json:"input"`
}

type Change struct {
	At      time.Time `json:"at"`
	Added   []string  `json:"added"`
	Removed []string  `json:"removed"`
}

type Catalogue struct {
	Updated time.Time `json:"updated"`
	Gems    []Gem     `json:"gems"`
	Changes []Change  `json:"changes"`
}

func fetchJSON(ctx context.Context, c *http.Client, url, ua string, out any) error {
	req, _ := http.NewRequestWithContext(ctx, "GET", url, nil)
	req.Header.Set("User-Agent", ua)
	resp, err := c.Do(req)
	if err != nil {
		return err
	}
	defer resp.Body.Close()
	if resp.StatusCode != 200 {
		return fmt.Errorf("%s: HTTP %d", url, resp.StatusCode)
	}
	return json.NewDecoder(resp.Body).Decode(out)
}

// fontEligible applies the exclusions poeringstash verified against the
// known pool sizes (red 40, green 75, blue 87 in 3.27/3.28):
//   - "... of Trarthus" gems are their own thing, not Font outcomes;
//   - "Vaal X (X of Y)" are corrupted variants;
//   - Imbued and support gems are not transfigured skill gems.
func fontEligible(name, base string) bool {
	return !strings.HasSuffix(name, " of Trarthus") &&
		!strings.HasPrefix(base, "Vaal ") &&
		!strings.Contains(name, "Imbued") &&
		!strings.HasSuffix(base, "Support")
}

func inputEligible(name string) bool {
	return !strings.HasSuffix(name, "Support") &&
		!strings.HasPrefix(name, "Vaal ") &&
		!strings.HasPrefix(name, "Awakened ") &&
		!strings.Contains(name, "Imbued")
}

func buildCatalogue(ctx context.Context, c *http.Client, ua string) ([]Gem, error) {
	var items struct {
		Result []struct {
			ID      string `json:"id"`
			Entries []struct {
				Type string `json:"type"`
				Text string `json:"text"`
				Disc string `json:"disc"`
			} `json:"entries"`
		} `json:"result"`
	}
	if err := fetchJSON(ctx, c, tradeItemsURL, ua, &items); err != nil {
		return nil, err
	}
	var repoe map[string]struct {
		Color    string `json:"color"`
		BaseItem *struct {
			DisplayName string `json:"display_name"`
		} `json:"base_item"`
	}
	if err := fetchJSON(ctx, c, repoeGemsURL, ua, &repoe); err != nil {
		return nil, err
	}
	colour := map[string]string{}
	for _, g := range repoe {
		if g.BaseItem != nil && g.BaseItem.DisplayName != "" && strings.Contains("rgb", g.Color) && g.Color != "" {
			colour[g.BaseItem.DisplayName] = g.Color
		}
	}
	var gems []Gem
	seen := map[string]bool{}
	for _, grp := range items.Result {
		if grp.ID != "gem" {
			continue
		}
		for _, e := range grp.Entries {
			if e.Type == "" {
				continue
			}
			g := Gem{Name: e.Type, Colour: colour[e.Type]}
			if e.Disc != "" {
				g.Name, g.Base = e.Text, e.Type
				g.Font = fontEligible(e.Text, e.Type)
			} else {
				g.Input = inputEligible(e.Type) && g.Colour != ""
			}
			if g.Name == "" || seen[g.Name] {
				continue
			}
			seen[g.Name] = true
			gems = append(gems, g)
		}
	}
	n := 0
	for _, g := range gems {
		if g.Font {
			n++
		}
	}
	// A shape change on either side would silently empty the pool; refuse
	// it and keep the previous catalogue instead.
	if n < 50 {
		return nil, fmt.Errorf("only %d Font gems found; the source format may have changed", n)
	}
	sort.Slice(gems, func(i, j int) bool { return gems[i].Name < gems[j].Name })
	return gems, nil
}

func fontSet(gems []Gem) map[string]bool {
	m := map[string]bool{}
	for _, g := range gems {
		if g.Font {
			m[g.Name] = true
		}
	}
	return m
}

// refreshCatalogue rebuilds it and records what changed in the Font pool.
func (s *Server) refreshCatalogue(ctx context.Context) error {
	gems, err := buildCatalogue(ctx, s.http, s.ua)
	if err != nil {
		return err
	}
	s.mu.Lock()
	defer s.mu.Unlock()
	old := s.cat
	cat := &Catalogue{Updated: time.Now(), Gems: gems}
	if old != nil {
		cat.Changes = old.Changes
		was, now := fontSet(old.Gems), fontSet(gems)
		var ch Change
		for g := range now {
			if !was[g] {
				ch.Added = append(ch.Added, g)
			}
		}
		for g := range was {
			if !now[g] {
				ch.Removed = append(ch.Removed, g)
			}
		}
		if len(ch.Added)+len(ch.Removed) > 0 {
			sort.Strings(ch.Added)
			sort.Strings(ch.Removed)
			ch.At = cat.Updated
			cat.Changes = append([]Change{ch}, cat.Changes...)
			if len(cat.Changes) > 20 {
				cat.Changes = cat.Changes[:20]
			}
		}
	}
	s.cat = cat
	b, _ := json.Marshal(cat)
	tmp := filepath.Join(s.dataDir, "catalogue.json.tmp")
	if err := os.WriteFile(tmp, b, 0o644); err == nil {
		os.Rename(tmp, filepath.Join(s.dataDir, "catalogue.json"))
	}
	return nil
}

func (s *Server) loadCatalogue() {
	b, err := os.ReadFile(filepath.Join(s.dataDir, "catalogue.json"))
	if err != nil {
		return
	}
	var cat Catalogue
	if json.Unmarshal(b, &cat) == nil && len(cat.Gems) > 0 {
		s.cat = &cat
	}
}
