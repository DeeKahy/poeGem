// poeGem: what is a Divine Font gem craft worth this league?
//
// The server only gathers data (the Font's gem pool, poe.ninja prices,
// leagues) and hands it to the page as one JSON document; the page does the
// maths, so every setting updates the numbers instantly.
package main

import (
	"context"
	"embed"
	"encoding/json"
	"flag"
	"io/fs"
	"log"
	"net/http"
	"os"
	"os/signal"
	"strings"
	"sync"
	"syscall"
	"time"
)

//go:embed web
var webFS embed.FS

const version = "2.0.0"

type Server struct {
	http    *http.Client
	ua      string
	dataDir string

	mu  sync.Mutex
	cat *Catalogue
	pc  priceCache

	leaguesMu sync.Mutex
	leagues   []string
	leaguesAt time.Time
}

func main() {
	addr := flag.String("addr", "127.0.0.1:8102", "listen address")
	dataDir := flag.String("data", envOr("POEGEM_DATA_DIR", "./data"), "where the catalogue is cached")
	flag.Parse()
	log.SetFlags(0)
	if err := os.MkdirAll(*dataDir, 0o755); err != nil {
		log.Fatal(err)
	}
	ua := "poegem/" + version
	if c := os.Getenv("POEGEM_CONTACT"); c != "" {
		ua += " (contact: " + c + ")"
	}
	s := &Server{
		http:    &http.Client{Timeout: 90 * time.Second},
		ua:      ua,
		dataDir: *dataDir,
		pc:      priceCache{leagues: map[string]*Prices{}, locks: map[string]*sync.Mutex{}},
	}
	s.loadCatalogue()

	ctx, stop := signal.NotifyContext(context.Background(), os.Interrupt, syscall.SIGTERM)
	defer stop()
	go s.catalogueLoop(ctx)

	static, _ := fs.Sub(webFS, "web")
	mux := http.NewServeMux()
	mux.Handle("GET /", http.FileServerFS(static))
	mux.HandleFunc("GET /healthz", func(w http.ResponseWriter, r *http.Request) { w.Write([]byte("ok\n")) })
	mux.HandleFunc("GET /api/leagues", s.handleLeagues)
	mux.HandleFunc("GET /api/data", s.handleData)

	hs := &http.Server{Addr: *addr, Handler: headers(mux), ReadHeaderTimeout: 10 * time.Second}
	go func() {
		<-ctx.Done()
		sh, cancel := context.WithTimeout(context.Background(), 5*time.Second)
		defer cancel()
		hs.Shutdown(sh)
	}()
	log.Printf("poegem %s on %s", version, *addr)
	if err := hs.ListenAndServe(); err != nil && err != http.ErrServerClosed {
		log.Fatal(err)
	}
}

func envOr(k, def string) string {
	if v := os.Getenv(k); v != "" {
		return v
	}
	return def
}

func headers(h http.Handler) http.Handler {
	return http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		w.Header().Set("X-Content-Type-Options", "nosniff")
		w.Header().Set("Content-Security-Policy", "default-src 'self'; img-src 'self' https://web.poecdn.com; frame-ancestors 'none'")
		h.ServeHTTP(w, r)
	})
}

// catalogueLoop rebuilds the Font pool at start (if stale) and daily.
func (s *Server) catalogueLoop(ctx context.Context) {
	for {
		s.mu.Lock()
		stale := s.cat == nil || time.Since(s.cat.Updated) > 24*time.Hour
		s.mu.Unlock()
		wait := time.Hour
		if stale {
			if err := s.refreshCatalogue(ctx); err != nil {
				log.Printf("catalogue: %v", err)
				wait = 10 * time.Minute
			} else {
				log.Printf("catalogue rebuilt")
			}
		}
		select {
		case <-ctx.Done():
			return
		case <-time.After(wait):
		}
	}
}

func writeJSON(w http.ResponseWriter, v any) {
	w.Header().Set("Content-Type", "application/json")
	json.NewEncoder(w).Encode(v)
}

// handleLeagues lists PC leagues from GGG, at most once every 6 hours.
func (s *Server) handleLeagues(w http.ResponseWriter, r *http.Request) {
	s.leaguesMu.Lock()
	defer s.leaguesMu.Unlock()
	if s.leagues == nil || time.Since(s.leaguesAt) > 6*time.Hour {
		var res struct {
			Result []struct{ ID, Realm string } `json:"result"`
		}
		if err := fetchJSON(r.Context(), s.http, "https://www.pathofexile.com/api/trade/data/leagues", s.ua, &res); err == nil {
			var ls []string
			for _, l := range res.Result {
				if l.Realm == "pc" {
					ls = append(ls, l.ID)
				}
			}
			if len(ls) > 0 {
				s.leagues, s.leaguesAt = ls, time.Now()
			}
		}
	}
	ls := s.leagues
	if ls == nil {
		ls = []string{"Standard"}
	}
	// The default is the current challenge league: the first that is not
	// Standard, Hardcore, Ruthless or SSF.
	def := ls[0]
	for _, l := range ls {
		low := strings.ToLower(l)
		if !strings.Contains(low, "standard") && !strings.Contains(low, "hardcore") && !strings.HasPrefix(low, "hc ") &&
			!strings.Contains(low, "ruthless") && !strings.Contains(low, "ssf") {
			def = l
			break
		}
	}
	writeJSON(w, map[string]any{"leagues": ls, "default": def})
}

// Wanted variants: the uncorrupted level/quality steps poe.ninja lists.
var variants = []string{"1", "1/20", "20", "20/20"}

func (s *Server) handleData(w http.ResponseWriter, r *http.Request) {
	league := r.URL.Query().Get("league")
	if league == "" {
		http.Error(w, "league required", 400)
		return
	}
	s.mu.Lock()
	cat := s.cat
	s.mu.Unlock()
	if cat == nil {
		http.Error(w, "the gem catalogue is still being built, try again in a minute", 503)
		return
	}
	ctx, cancel := context.WithTimeout(r.Context(), 60*time.Second)
	defer cancel()
	p, err := s.prices(ctx, league)
	if err != nil {
		http.Error(w, "prices: "+err.Error(), 502)
		return
	}
	type outGem struct {
		Gem
		Q map[string]Quote `json:"q,omitempty"`
	}
	var gems []outGem
	uncoloured := 0
	for _, g := range cat.Gems {
		if !g.Font && !g.Input {
			continue
		}
		if g.Colour == "" {
			if c := p.IconColour[g.Name]; c != "" {
				g.Colour = c
			} else if c := p.IconColour[g.Base]; c != "" {
				g.Colour = c
			} else if g.Font {
				uncoloured++
			}
		}
		og := outGem{Gem: g}
		if q := p.Gems[g.Name]; q != nil {
			og.Q = map[string]Quote{}
			for _, v := range variants {
				if x, ok := q[v]; ok {
					og.Q[v] = x
				}
			}
		}
		gems = append(gems, og)
	}
	w.Header().Set("Cache-Control", "public, max-age=300")
	writeJSON(w, map[string]any{
		"league":     league,
		"prices_at":  p.Fetched,
		"catalogue":  cat.Updated,
		"changes":    cat.Changes,
		"divine":     p.Divine,
		"offerings":  p.Offers,
		"uncoloured": uncoloured,
		"gems":       gems,
	})
}
