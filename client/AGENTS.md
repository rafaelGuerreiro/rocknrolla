# Client guidance

This file extends the repository root `AGENTS.md` for work under `client/`.

## Scope

- Build gameplay with TypeScript, Phaser, and Phaser's bundled Matter physics.
- Keep physics, input, rendering, obstacle handling, and waypoint detection client-side.
- Use pointer input so mouse and touch follow the same path.
- Keep the Vite web build as the source consumed by Capacitor.

## Working rules

- Prefer direct Phaser scene code and browser APIs over managers, plugins, UI frameworks, or standalone `matter-js`.
- Keep tuning constants near the scene or object that owns them.
- Do not edit `src/module_bindings/` by hand; regenerate it with `task server:sdk-ts` from the repository root.
- Treat `android/` and `ios/` as native source projects. Do not commit copied web assets, local SDK paths, signing keys, or build output.
- Add native plugins only for a current device capability.

## Commands

- `task dev`: run Vite.
- `task build`: type-check and build the web client.
- `task sync`: build and sync both Capacitor projects.
- `task ios` / `task android`: sync and open the native IDE.

Run `task build` after client changes. Run `task sync` when Capacitor configuration, plugins, or native integration changes.
