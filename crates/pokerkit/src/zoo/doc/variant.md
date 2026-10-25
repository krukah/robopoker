The cube grain — single shape used everywhere the user picks a bot.
Hosting endpoint, slumbot CLI, and the closed-source gameplay UI all parse into the same
[`Variant`]; the cube IS the identity (`bot:<label>` username + derived UUID
v5), with no name layer in between.

Pure data lives here in [`pokerkit`] so a WASM frontend can use it without
dragging in the database/nlhe feature graph. Gameroom-side extensions
(`into_player`, `member`, telemetry keys) live in
`parlor::players::roster::variant` and read this type.

Wire format (serde):

```json
{ "kind": "fish" }
{ "kind": "bot", "depth": false, "world": true, "dirac": false }
```

CLI grammar:

- `fish` — random opponent (no model, not in the cube)
- `base` — the empty flag-set cube cell (raw blueprint sample)
- `+`-joined flags from `{depth, world, dirac}` in canonical order:
  `depth`, `world`, `dirac`, `depth+world`, `depth+dirac`,
  `world+dirac`, `depth+world+dirac`
