<h1 align="center">espnclock</h1>

<p align="center"><strong>Unofficial ESPN scoreboard CLI — a typed game clock, not an SDK.</strong></p>

<p align="center">
  <a href="https://github.com/copyleftdev/espnclock/actions/workflows/ci.yml"><img src="https://github.com/copyleftdev/espnclock/actions/workflows/ci.yml/badge.svg" alt="CI"></a>
  <a href="LICENSE"><img src="https://img.shields.io/badge/license-MIT-blue.svg" alt="MIT License"></a>
  <a href="https://www.rust-lang.org"><img src="https://img.shields.io/badge/rust-1.80%2B-orange" alt="Rust 1.80+"></a>
  <img src="https://img.shields.io/badge/API-unofficial%20site.v2-lightgrey" alt="Unofficial ESPN site.v2 JSON">
</p>

<p align="center">NFL · NBA · MLB · NHL · NCAAF · NCAAB · WNBA · UFC</p>

---

**espnclock** reads ESPN's public `site.v2` scoreboard JSON (no API key) and
exposes one fact: has this event started?

`pre` / `in` / `post` · UTC commence · ESPN abbreviations · local SSE fan-out.

It is **not** an ESPN SDK, not a fantasy client, not a odds feed, and not
affiliated with ESPN. There is no official public ESPN sports API.

```
$ espnclock board nfl
in   MIA  @ BUF   1:45 - 2nd   2026-11-22T18:00Z  401873070
pre  LV   @ DEN   1:25 PM PST  2026-11-22T21:25Z  401873075

$ espnclock status nfl LV DEN --json
$ espnclock started nfl          # exit 0/2/3

$ espnclock serve                # 127.0.0.1:8796
$ curl -s http://127.0.0.1:8796/health
$ curl -Ns http://127.0.0.1:8796/events
```

## Why a clock

Maker bids on event contracts should die at kickoff. ESPN already knows
`status.type.state`. Sharp books know the line. Mixing those jobs is how
tools get slow and wrong. espnclock only does the clock.

## Serve (one poller, many listeners)

ESPN has **no WebSocket**. `serve` is *your* socket:

- One process. ≥15s between any ESPN fetches (their `Cache-Control` is ~2s).
- Idle leagues every 5 minutes. Hot = `in`, or `pre` within 2 hours of commence.
- N local clients on SSE (`/events`). They do not multiply ESPN hits.
- Bind defaults to `127.0.0.1`. Non-loopback requires `--expose`.

Do not put N Cloudflare Workers in front of this. That is the tax.

See [docs/serve.md](docs/serve.md).

## ESPN surfaces (read this)

| Host | Shape | This crate |
|------|--------|------------|
| `site.api.espn.com/apis/site/v2/sports/{sport}/{league}/scoreboard` | Inline events, `max-age` 1–4s | **pinned** |
| `sports.core.api.espn.com/v2/...` | Paginated `{items:[{$ref}]}` | out of scope |
| `site.web.api.espn.com` | Personalized / header | out of scope |

Pinned fields: `events[].date`, `status.type.state` ∈ `{pre,in,post}`,
`competitions[0].competitors[].homeAway` + `team.abbreviation`.

Details: [docs/hosts.md](docs/hosts.md).

## Intricacies

- **Unofficial.** ESPN can 403 or reshape without notice. Feeders should
  fail-open (keep last good clock).
- **Abbreviations are ESPN's.** Washington is `WSH`, not Kalshi `WAS`. Map
  at the consumer. Jacksonville is `JAX`.
- **`dates=`** is `YYYYMMDD`. Omit for ESPN's "today".
- **UFC** is an MMA card, not a home/away blob. `state` still works.
- **User-Agent** is sent. Empty UA sometimes 403s.
- **Not for resale** of ESPN data.

## Install

```
git clone https://github.com/copyleftdev/espnclock
cd espnclock
cargo install --path .
```

Copy `target/release/espnclock` to `~/.local/bin` if you skip `cargo install`.

## Commands

| Command | Meaning |
|---------|---------|
| `espnclock leagues` | nfl ncaaf nba wnba ncaab mlb nhl ufc |
| `espnclock board nfl` | full board |
| `espnclock started nfl` | state ≠ pre |
| `espnclock status nfl LV DEN` | one matchup (order-insensitive) |
| `espnclock serve` | one poller, local SSE |
| `--json` | machine output |
| `--date 20261122` | pin the board day |
| `--expose` | allow non-loopback bind |

Exit codes: `0` ok · `1` bad args · `2` ESPN/network · `3` matchup not on board.

## Mesh / trading

Not wired. Test the clock alone first. Proposed later use:
[docs/integration.md](docs/integration.md).

## License

MIT. ESPN names and data remain ESPN's.
