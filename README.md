# poeGem

What a Divine Font gem craft is worth in the lab, this league. Live at
<https://poegem.smallapp.cc>.

## What it answers

**Random transfigure, pick 1 of 3.** You put in a gem; the Font offers three
different transfigured gems of its colour and you keep the most valuable. For
each colour the page shows the expected value of the gem you keep, the cheapest
gem of that colour to put in, the net per craft, the chance at least one of the
three is worth keeping, and a per-lab-run total. Click a colour for every
possible outcome with its chance of being the one you keep.

**Transform into its own transfigured version.** This option only shows up
sometimes. When it does, the gem worth having brought is the one whose best
transfigured form is worth the most above the base gem. The page shows the top 3
and a full searchable table.

Settings: league, level/quality of the gem you put in (1, 1/20, 20, 20/20),
ignore results under N chaos or with fewer than N listings (they still take a
slot, they just count as 0), crafts per run, and lab cost. They live in the URL,
so a link reproduces the view.

## The maths

Sort the colour's pool best-first. Gem `r` (0-based) is the one you keep exactly
when the other two offered gems both rank below it:

    P(r) = C(n - r - 1, 2) / C(n, 3)        EV = Σ value(r) · P(r)

The probabilities sum to 1, and the result matches a brute force over every
triple. It assumes every eligible gem is equally likely; GGG publishes no weights.

## Where the gem list comes from (and why it follows new seasons)

Nothing is hand-maintained. Once a day the server rebuilds the Font pool from:

- GGG's trade catalogue (`/api/trade/data/items`): a transfigured gem has a
  discriminator and its `type` is the base gem. The old version guessed from
  `" of "` in the name, which counted Herald of Ash, Orb of Storms, Purity of Ice
  and others as lab outcomes, and missed any gem poe.ninja did not list.
- RePoE for base gem colours (a transfigured gem has its base gem's colour), with
  the colour baked into GGG's icon URL as a fallback for brand-new gems.

Excluded: Trarthus gems, Vaal variants, supports and Imbued gems. In 3.28 this
gives red 40, green 75, blue 87. When a patch adds or removes gems, the page
lists the change.

Prices: poe.ninja (`/poe1/api/economy/...`), cached 30 minutes per league.

## Run

    nix develop
    go test ./...
    go run . -addr 127.0.0.1:18102 -data ./data

NixOS: `services.poegem = { enable = true; port = 8102; };` from this flake's
`nixosModules.default`. Standard library only, no database.

The previous Rust version is in the git history.
