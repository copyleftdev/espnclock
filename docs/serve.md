# serve

Poll ESPN from one process. Serve the snapshot on localhost.

```
espnclock serve --bind 127.0.0.1:8796 --interval 15
```

| Path | Type | Body |
|------|------|------|
| `/health` | JSON | `ok`, `fetches`, `leagues_hot`, `clients` |
| `/clock` | JSON | `{fetches, leagues_hot, boards}` |
| `/events` | SSE | `event: clock` when a board fingerprint changes; `: keepalive` |

## Rate

- `--interval` is a floor (default 15s, cannot go lower) between **any** two
  ESPN fetches, across all leagues.
- A league with no live or imminent games is fetched every 300s.
- Imminent = `pre` and start time within 2 hours. Live = `in`. Finals (`post`)
  are idle.
- At most 16 SSE clients.

Many local clients on `/events` still mean one ESPN client.

## Bind

`--bind 0.0.0.0:8796` is refused unless you pass `--expose`. This is not meant
to be an open relay for ESPN's JSON.
