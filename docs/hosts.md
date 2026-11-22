# ESPN HTTP surfaces

There is no official public ESPN sports API. Three hosts show up in the wild.
espnclock uses one of them.

## site.api.espn.com (used)

```
GET https://site.api.espn.com/apis/site/v2/sports/{sport}/{league}/scoreboard
GET .../scoreboard?dates=YYYYMMDD
```

- Inline `events[]`. No `$ref`.
- `Cache-Control: max-age=1..4` (NFL scoreboard measured ~2–3s).
- `User-Agent` required in practice.
- No key.

| slug | path |
|------|------|
| nfl | football/nfl |
| ncaaf | football/college-football |
| nba | basketball/nba |
| wnba | basketball/wnba |
| ncaab | basketball/mens-college-basketball |
| mlb | baseball/mlb |
| nhl | hockey/nhl |
| ufc | mma/ufc |

Fields we depend on:

- `events[].id`
- `events[].date` — ISO UTC; sometimes without seconds (`…T21:25Z`)
- `events[].status.type.state` — `pre` \| `in` \| `post`
- `events[].status.type.shortDetail`
- `events[].competitions[0].competitors[].homeAway`
- `events[].competitions[0].competitors[].team.abbreviation`
- `events[].competitions[0].competitors[].score` (string)

## sports.core.api.espn.com (not used)

Paginated `{ count, items: [ { "$ref": "http://…" } ] }`. Following `$ref` is
a different program.

## site.web.api.espn.com (not used)

Personalized header feeds. Not a stable public contract.

## Out of scope

Play-by-play, odds, fantasy, athlete graphs.
