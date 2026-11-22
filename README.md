<h1 align="center">espnclock</h1>

<p align="center"><strong>A small CLI for ESPN scoreboard status. Unofficial. No API key.</strong></p>

<p align="center">
  <a href="https://github.com/copyleftdev/espnclock/actions/workflows/ci.yml"><img src="https://github.com/copyleftdev/espnclock/actions/workflows/ci.yml/badge.svg" alt="CI"></a>
  <a href="LICENSE"><img src="https://img.shields.io/badge/license-MIT-blue.svg" alt="MIT License"></a>
  <a href="https://www.rust-lang.org"><img src="https://img.shields.io/badge/rust-1.80%2B-orange" alt="Rust 1.80+"></a>
</p>

<p align="center">NFL · NBA · MLB · NHL · NCAAF · NCAAB · WNBA · UFC</p>

---

espnclock prints whether a game has started, when it is scheduled, and the
ESPN team abbreviations. That is the whole product.

ESPN does not publish a public sports API. This talks to the same JSON the
ESPN website already loads (`site.api.espn.com` … `/scoreboard`). No key.
Not affiliated with ESPN.

```
$ espnclock board nfl
in   MIA  @ BUF   1:45 - 2nd   2026-11-22T18:00Z  401873070
pre  LV   @ DEN   1:25 PM PST  2026-11-22T21:25Z  401873075

$ espnclock status nfl LV DEN --json
$ espnclock started nfl
```

`state` is `pre`, `in`, or `post`.

## Install

```
git clone https://github.com/copyleftdev/espnclock
cd espnclock
cargo install --path .
```

Requires Rust 1.80+. The binary is `espnclock`.

## Commands

| Command | What it does |
|---------|----------------|
| `espnclock leagues` | list slugs |
| `espnclock board nfl` | today's board |
| `espnclock started nfl` | games that have left `pre` |
| `espnclock status nfl LV DEN` | one matchup (either order) |
| `espnclock serve` | local HTTP + SSE (see below) |
| `--json` | JSON instead of columns |
| `--date 20261122` | pin the calendar day (`YYYYMMDD`) |

Exit codes: `0` ok · `1` bad args · `2` network/ESPN · `3` matchup not on the board.

## `serve` — one poller, many local readers

ESPN has no public WebSocket. `serve` polls on a polite timer and fans out
to whatever is listening on your machine.

```
$ espnclock serve                 # 127.0.0.1:8796
$ curl -s http://127.0.0.1:8796/health
$ curl -s http://127.0.0.1:8796/clock
$ curl -Ns http://127.0.0.1:8796/events
```

- At least 15 seconds between any two ESPN fetches.
- Quiet leagues: every 5 minutes. Active: `in`, or `pre` within 2 hours of start.
- Defaults to localhost. `--bind 0.0.0.0:…` needs `--expose`.

Full paths: [docs/serve.md](docs/serve.md).

## What it is not

Not an ESPN SDK. Not fantasy. Not odds. Not play-by-play. It does not wrap
`sports.core.api.espn.com` (`$ref` pagination) or `site.web.api.espn.com`.

Which host, which JSON fields: [docs/hosts.md](docs/hosts.md).

## Caveats

- Unofficial. ESPN can change or block the JSON without notice.
- Team codes are ESPN's (`WSH` for Washington, `JAX` for Jacksonville).
- Send a User-Agent (the binary does). Empty UA can 403.
- Do not hammer the endpoint. Do not run this as a public ESPN proxy.
- Do not resell ESPN data.

## License

MIT. ESPN names and data remain ESPN's.
