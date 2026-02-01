# POE Gem Calculator

A tool to figure out which transfigured skill gem color gives the best expected return in Path of Exile. Pulls pricing data from [POE Ninja](https://poe.ninja/), groups gems by color, and calculates the average value for each.

## What it does

When you transfigure a skill gem in the lab, you get a random transfigured version. This tool tells you which gem color (red/green/blue) has the highest expected value based on current market prices.

You can filter by:
- League
- Gem level (1, 20, or 21)
- Gem quality (0, 20, or 23)
- Minimum chaos value (ignores cheap gems that won't sell)

## Running it

### Docker

```bash
docker-compose up -d
```

Then open http://localhost:3000

### From source

```bash
cargo build --release
cargo run
```

### CLI options

```
-p, --port <PORT>       Port [default: 3000]
    --host <HOST>       Host [default: 0.0.0.0]
    --cache-dir <DIR>   Cache directory [default: cache]
    --log-level <LEVEL> Log level [default: info]
```

## API

- `GET /` - Web UI
- `GET /health` - Health check
- `GET /api/leagues` - List available leagues
- `GET /api/skill-gems?league=<league>` - Raw gem data
- `GET /api/calculate?league=<league>&ignore_after_chaos=5&gem_level=1&gem_quality=0` - Calculate best color

## Project structure

```
src/
  main.rs           # Server setup
  api/
    leagues.rs      # League endpoints
    skill_gems.rs   # Gem data and calculation
  cache/
    mod.rs          # File-based caching
  models/
    mod.rs          # Data types
```

## Notes

This depends on POE Ninja's API. If their format changes, things will break. Open an issue or PR if that happens.

Data is cached for 1 hour to avoid hammering their API.

## License

MIT