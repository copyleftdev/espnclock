# ESPN HTTP surfaces

There is no official public ESPN sports API. Three hosts show up in the wild.

## site.api.espn.com (pinned)

```
GET https://site.api.espn.com/apis/site/v2/sports/{sport}/{league}/scoreboard
GET .../scoreboard?dates=YYYYMMDD
```

- Inline `events[]`. No `$ref`.
- `Cache-Control: max-age=1..4` (NFL scoreboard measured ~2–3s).
- `User-Agent` required in practice.
- No key.

Sport/league pairs espnclock knows:

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

Contract we pin:

- `events[].id`
- `events[].date` — ISO UTC, sometimes without seconds (`...T21:25Z`)
- `events[].status.type.state` — `pre` | `in` | `post`
- `events[].status.type.shortDetail`
- `events[].competitions[0].competitors[].homeAway`
- `events[].competitions[0].competitors[].team.abbreviation`
- `events[].competitions[0].competitors[].score` (string)

## sports.core.api.espn.com (out of scope)

Paginated `{ count, items: [ { "$ref": "http://..." } ] }`. Following `$ref`
is a crawler, not a clock. Cache-Control on `/v2/sports` measured `max-age=900`.

## site.web.api.espn.com (out of scope)

Personalized / scoreboard header. Varnish. Not a stable public contract.

## What we will not add

Play-by-play, odds, fantasy, athlete graphs, Cloudflare worker farms.
