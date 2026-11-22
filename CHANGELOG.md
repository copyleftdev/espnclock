# Changelog

## 0.2.0 — 2026-11-22

- `espnclock serve`: one ESPN poller, localhost SSE fan-out (`/events`, `/clock`, `/health`).
- Global floor 15s between fetches. Idle leagues 300s. Hot = `in` or `pre` within 2h of commence.
- Bind defaults to `127.0.0.1`. Non-loopback requires `--expose`.

## 0.1.0 — 2026-11-22

- First cut: site.v2 scoreboard clock (`pre`/`in`/`post`).
- Leagues: nfl, ncaaf, nba, wnba, ncaab, mlb, nhl, ufc.
- Commands: `leagues`, `board`, `started`, `status`.
- `--json`, `--date YYYYMMDD`.
- Fixture parse tests. Live ESPN is not part of CI.
