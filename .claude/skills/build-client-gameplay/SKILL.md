---
name: build-client-gameplay
description: Implement or diagnose RocknRolla gameplay in the Phaser TypeScript client. Use for Phaser scenes, Matter physics, touch or pointer input, obstacles, levels, cameras, rendering, client waypoint reporting, Vite, or Capacitor integration under client/. Do not use for server-only progression or build automation.
---

# Build Client Gameplay

Build the smallest playable browser-first change and keep it compatible with Capacitor.

## Workflow

1. Read `AGENTS.md`, `client/package.json`, and the affected client files.
2. Trace the current Phaser scene, Matter bodies, input handlers, and lifecycle before editing.
3. Reuse the current scene and direct Phaser APIs before adding classes, managers, plugins, or state frameworks.
4. Keep movement, collisions, escape input, and waypoint detection in the client.
5. Report only discrete waypoints or outcomes to SpacetimeDB. Do not stream transforms or ask the server to validate physics.
6. Run `task client:fmt`, `task client:lint`, and `task client:build`, then exercise the affected behavior in a browser. Run `task client:sync` when native files or configuration change.

## Defaults

- Use Phaser's bundled Matter integration; do not install standalone `matter-js` without a concrete need outside Phaser.
- Use pointer events so mouse and touch share one input path.
- Use browser and Phaser primitives before adding UI, state, or asset libraries.
- Keep tuning constants near the scene or object that owns them.
- Keep the web build as the source for Capacitor; do not duplicate gameplay in native code.
- Do not add prediction, reconciliation, rollback, anti-cheat, or server-authoritative movement.

When verification cannot run, state which executable, native platform, or environment is missing.
