# Security

espnclock only **reads** a public, unauthenticated JSON URL. It stores no
credentials and does not write to ESPN.

- Do not point it at authenticated ESPN cookies.
- Default bind is loopback. `--expose` is required for a non-loopback bind.
- Report issues at https://github.com/copyleftdev/espnclock/issues
