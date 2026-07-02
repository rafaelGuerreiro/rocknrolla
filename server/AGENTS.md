# Server guidance

This file extends the repository root `AGENTS.md` for work under `server/`.

## Scope

- Keep deployable SpacetimeDB modules in `bins/*` and reusable Rust crates in `sdks/*`.
- Store rewards and progression; accept client-reported waypoints without simulating physics.
- Use `ctx.sender()` for ownership and authentication. Never trust an identity supplied as reducer input.

## SpacetimeDB rules

- Keep reducers deterministic and transactional: no filesystem, network, external timers, or nondeterministic randomness.
- Read state through tables and subscriptions; reducers mutate state and do not return query data.
- Make tables private unless clients need to subscribe.
- Add indexes only for current queries.
- Return `Result<(), String>` for expected failures; do not panic on client input.
- Keep `Cargo.lock` committed for reproducible module builds.
- Do not edit generated TypeScript bindings by hand.

## Commands

- `task check`: format and check all Rust targets.
- `task test`: run Clippy and tests.
- `task sdk-ts`: regenerate client bindings.
- `task build`: regenerate bindings and build `rocknrolladb` for WASM.
- `task publish`: publish the development database.

Run `task check` after Rust changes, `task test` for behavior changes, and `task build` when the public module interface changes.
