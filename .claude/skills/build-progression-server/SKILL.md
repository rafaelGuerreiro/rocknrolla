---
name: build-progression-server
description: Implement or diagnose RocknRolla rewards and progression in the Rust SpacetimeDB server. Use for tables, reducers, subscriptions, identities, waypoint claims, reward collection, progress persistence, generated bindings, or SpacetimeDB commands under server/. Do not use for client physics or packaging.
---

# Build Progression Server

Build the smallest transactional progression change. SpacetimeDB is the server and database; do not introduce a separate API layer.

## Workflow

1. Read `AGENTS.md`, the Rust workspace manifests, and the affected module, tables, reducers, and generated bindings.
2. Model only state required by the current reward or progress behavior.
3. Add or change a reducer for each client mutation and a public table or view only when the client must subscribe to it.
4. Resolve `ctx.sender()` in the reducer as ownership identity, then pass it into the service as an `Identity` parameter. Never trust a player identity supplied as a reducer argument.
5. Accept client-reported waypoint or completion claims as valid. Validate shape, ownership, duplicates, and impossible database transitions, but do not emulate physics or add anti-cheat.
6. Update the Phaser client call or subscription when the feature spans both sides, then regenerate bindings instead of editing generated files.
7. Run `task server:lint` (fmt + Clippy, denying warnings across the whole workspace — not just the files you touched), the narrowest relevant Rust tests, and `task server:build` when bindings or the module interface change.

## SpacetimeDB rules

- Keep reducers deterministic and transactional: no filesystem, network, external timers, or nondeterministic randomness.
- Read results through subscriptions; reducers mutate state and do not return query data.
- Use current Rust APIs: `&ReducerContext`, `ctx.sender()`, `ctx.db.table()`, and the `Table` trait.
- Make tables private unless clients need to subscribe.
- Add indexes only for current query patterns.
- Return `ServiceResult<()>` (from `sdks/rocknrolla-error`, re-exported at `src/error.rs`) for expected failures.
- Use one straightforward table and reducer before decomposing schemas or introducing helpers.
- Do not add REST services, caches, queues, telemetry pipelines, migrations, or server-side physics.

## Domain boundaries

- Table structs hold data only: no `impl` blocks beyond derives, no methods. Behavior lives in that domain's `services` module.
- A domain's `*Services` may only touch tables defined in its own `{domain}.rs`. Every cross-domain read or write goes through the other domain's `*ReducerContext` accessor (e.g. `self.character_services().piece_exists(..)`), never a direct `self.db.<other_domain_table>()` call.
- Reducers only validate parameters, resolve `ctx.sender()`, and delegate to one service call; they never touch `ctx.db` themselves.
- Pass `Identity` into service methods as an explicit parameter rather than calling `ctx.sender()` inside a service, so the method stays reusable when the acting identity isn't the caller.

## Error handling

- Never write `.unwrap()`, `.expect(...)`, `panic!`, `unreachable!`, or `todo!` outside `#[cfg(test)]` code.
- Prefer `ServiceError` over a panic even when a failure looks structurally impossible (e.g. reading back a row you just inserted) — the invariant can drift later, and a typed error costs nothing extra to write.
- `ServiceError` lives in `sdks/rocknrolla-error`, not this module — it has no SpacetimeDB dependency, so other sdk crates (e.g. `rocknrolla-level`) can return it too. Reuse it there when a helper's failure fits one of its categories (validation, not-found, conflict, forbidden, internal); keep a crate-local typed error (like `rocknrolla-level`'s `CodecError`) when callers pattern-match on specific variants. `bins/admin` is a separate leaf CLI and uses `anyhow` instead, since it isn't depended on by anything else.

When verification cannot run, state which manifest, module, binding target, or executable is missing.
