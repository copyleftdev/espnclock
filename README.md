<h1 align="center">espnclock</h1>

<p align="center"><strong>Typed event clock over ESPN's public site.v2 scoreboard JSON.</strong></p>

<p align="center">
  <a href="https://github.com/copyleftdev/espnclock/actions/workflows/ci.yml"><img src="https://github.com/copyleftdev/espnclock/actions/workflows/ci.yml/badge.svg" alt="CI"></a>
  <a href="LICENSE"><img src="https://img.shields.io/badge/license-MIT-blue.svg" alt="MIT License"></a>
  <a href="https://www.rust-lang.org"><img src="https://img.shields.io/badge/rust-1.80%2B-orange" alt="Rust 1.80+"></a>
</p>

---

**espnclock** is not an ESPN SDK and not a fantasy client. It is a **clock**:
`pre` / `in` / `post`, commence time, and abbreviations — the one fact a
maker bid needs (has this game started?).

There is **no official public ESPN sports API**. This talks to the same JSON
the ESPN website already serves, with no key.

```
$ espnclock board nfl
in   MIA  @ BUF   1:45 - 2nd  2026-11-22T18:00Z  401873070
pre  LV   @ DEN   1:25 PM PST 2026-11-22T21:25Z  401873075

$ espnclock status nfl LV DEN --json
$ espnclock started nfl
$ echo $?   # 0 found, 2 ESPN down, 3 no such matchup

$ espnclock serve                  # 127.0.0.1:8796
$ curl -s http://127.0.0.1:8796/health
$ curl -Ns http://127.0.0.1:8796/events
```

`serve` is the socket. **One** process polls ESPN (≥15s between any fetches, idle leagues every 5 min, only `in` or `pre`≤2h are hot). N local clients subscribe over SSE. ESPN still sees HTTP. There is no ESPN WebSocket; do not put N Cloudflare Workers in front of this.

## What this is (and is not)

ESPN exposes at least three HTTP surfaces:

| Host | Shape | espnclock |
|------|--------|-----------|
| `site.api.espn.com/apis/site/v2/sports/{sport}/{league}/scoreboard` | Inline events, `max-age` 1–4s | **v0.1 — this** |
| `sports.core.api.espn.com/v2/...` | Paginated `{items:[{$ref}]}` | out of scope |
| `site.web.api.espn.com` | Personalized / header feeds | out of scope |

v0.1 does **not** wrap core `$ref` graphs, play-by-play, odds, or fantasy.
Those are a different product. The scoreboard contract is the one that is
stable enough to pin: `events[].date`, `status.type.state` ∈ `{pre,in,post}`,
`competitions[0].competitors[].homeAway` + `team.abbreviation`.

## Intricacies (read before depending)

- **Unofficial.** ESPN can 403, reshape, or cache-bust this without notice.
  Treat fetch errors as fail-open in trading feeders (keep last good clock).
- **No SLA.** Scoreboard `Cache-Control` is ~1–4 seconds. Do not hammer it.
- **Abbreviations are ESPN's.** Washington is `WSH`, not Kalshi `WAS`. Map
  at the consumer. Jacksonville is `JAX`.
- **`dates=`** is `YYYYMMDD` (or a year). Omit it for "today's board" as ESPN
  defines today.
- **UFC cards** are events on the MMA scoreboard, not NFL-style home/away
  blobs. `status` still works; pair matching is weaker.
- **User-Agent** is sent. Some CDNs 403 empty UA.
- **Not affiliated with ESPN.** Not for resale of ESPN data.

## Installation

```
git clone https://github.com/copyleftdev/espnclock
cd espnclock
cargo install --path .
```

Binary: `espnclock` on PATH (`~/.cargo/bin` or copy to `~/.local/bin`).

## Commands

| Command | Meaning |
|---------|---------|
| `espnclock leagues` | slugs: nfl ncaaf nba wnba ncaab mlb nhl ufc |
| `espnclock board nfl` | full board |
| `espnclock started nfl` | state ≠ pre |
| `espnclock status nfl LV DEN` | one matchup (order-insensitive) |
| `espnclock serve [--bind 127.0.0.1:8796] [--interval 15]` | one poller, local SSE |
| `--json` | machine output |
| `--date 20261122` | pin the board day |

Exit codes: `0` ok, `1` bad args, `2` ESPN/network, `3` matchup not on board.

## License

MIT. ESPN names and data remain ESPN's.
