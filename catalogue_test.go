package main

import "testing"

func TestFontEligible(t *testing.T) {
	cases := []struct {
		name, base string
		want       bool
	}{
		{"Arc of Oscillating", "Arc", true},
		{"Bladefall of Trarthus", "Bladefall", false},
		{"Vaal Arc (Arc of Oscillating)", "Vaal Arc", false},
	}
	for _, c := range cases {
		if got := fontEligible(c.name, c.base); got != c.want {
			t.Errorf("%s: got %v", c.name, got)
		}
	}
	// Base gems with " of " in the name are inputs, never outcomes: they have no disc.
	if !inputEligible("Herald of Ash") || inputEligible("Added Fire Damage Support") || inputEligible("Awakened Spell Echo Support") {
		t.Error("input rules")
	}
}

func TestIconColour(t *testing.T) {
	red := "https://web.poecdn.com/gen/image/WzMwLDE0LHsiZiI6IjJESXRlbXMvR2Vtcy9Nb2x0ZW5TdHJpa2UiLCJ3IjoxLCJoIjoxLCJzY2FsZSI6MSwiZ2QiOjV9XQ/b3e2ef9d6c/MoltenStrike.png"
	if c := iconColour(red); c != "r" {
		t.Fatalf("got %q", c)
	}
	if iconColour("https://example.com/x.png") != "" {
		t.Fatal("non-gem icon")
	}
}

func TestVariant(t *testing.T) {
	if variant(20, 20) != "20/20" || variant(1, 0) != "1" {
		t.Fatal(variant(20, 20), variant(1, 0))
	}
}
