# Later integration (not wired)

espnclock is tested as a clock first. Do not import it into a trading mesh
until a paper window shows it changes outcomes.

## What it can do for a Kalshi cycle

Cycle rests 1c under a sportsbook fair. Unfilled bids into kickoff become
in-play inventory. ESPN `state` is the kickoff bit:

- `espnclock status nfl LV DEN` exit 0 + `"state":"in"` → drop the ticker
  from `sports_anchors.json` so the mesh cancels (`!anchors.contains_key`).
- Fail-open: ESPN 403/timeout → keep last fair (TheRundown 75m buffer remains).
- Map abbreviations at the feeder (`WSH` → `WAS`). Never inside espnclock.

## What it must not do

- Replace TheRundown de-vig fairs.
- Call Jev (no sports skill).
- Poll from kalshismesh's 10s loop (would break the 15s ESPN floor).
- Talk to ESPN from N mesh nodes.

## Proposed seam (when we test)

`sports_cycle_anchors.py` already filters ESPN via urllib. Optional later:
subprocess `espnclock started nfl --json` or `GET http://127.0.0.1:8796/clock`
if `serve` is up. One poller either way.

Paper test: compare a Sunday of cycle cancels vs TheRundown `event_date+75m`
alone. Benefit = fewer in-play rests, not extra fills.
