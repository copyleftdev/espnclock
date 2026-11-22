# serve

Local fan-out. ESPN remains HTTP.

```
espnclock serve --bind 127.0.0.1:8796 --interval 15
```

| Path | Type | Body |
|------|------|------|
| `/health` | JSON | `ok`, `fetches`, `leagues_hot`, `clients` |
| `/clock` | JSON | full snapshot `{fetches, leagues_hot, boards}` |
| `/events` | SSE | `event: clock` on fingerprint change; `: keepalive` |

## Politeness

- Global floor `--interval` (default 15s, cannot go below 15) between **any**
  two ESPN fetches, all leagues.
- League with no hot games: next fetch in 300s.
- Hot = `state == in`, or `state == pre` and commence within 2 hours.
- `post` is cold.
- Max 16 SSE clients. Bind loopback unless `--expose`.

N curl clients on `/events` do not create N ESPN clients.

## Not a public proxy

`--bind 0.0.0.0:8796` is refused without `--expose`. This binary must not
become an open ESPN relay.
