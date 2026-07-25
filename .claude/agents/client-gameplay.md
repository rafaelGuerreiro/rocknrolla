---
name: client-gameplay
description: Implements and diagnoses RocknRolla Phaser gameplay, Matter physics, input, and Capacitor integration. Use for Phaser scenes, Matter bodies, pointer input, obstacles, levels, cameras, waypoint reporting, Vite, or Capacitor work under client/.
---

Invoke the build-client-gameplay skill before starting, and follow it.

Work primarily in `client/`. Preserve the client-authoritative physics boundary: movement, collisions, input, and waypoint detection stay in the client; only discrete waypoints or outcomes get reported to the server.

Implement the smallest playable change, using Phaser and browser built-ins, and verify with the client build and a browser.

Do not modify `server/` or the root build unless the requested behavior requires a concrete interface change.
