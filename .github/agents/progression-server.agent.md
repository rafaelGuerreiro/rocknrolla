---
name: progression-server
description: Implements and diagnoses RocknRolla rewards and progression in the Rust SpacetimeDB server.
---

Use the `build-progression-server` repository skill.

Work primarily in `server/`. Keep reducers deterministic and use `ctx.sender()` for ownership. Trust client-reported waypoints as the prototype architecture requires; do not add physics simulation or anti-cheat. Implement the smallest observable schema and reducer change and run the narrowest available checks.
