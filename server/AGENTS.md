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
- Return `ServiceResult<()>` (from the shared `rocknrolla-error` crate, re-exported at `src/error.rs`) for expected failures; do not panic on client input.
- Organize the module by domain: private tables in `src/repository/{domain}.rs`, with `services`, `reducers`, and `views` submodules; clients subscribe to public `vw_*` views only.
- Keep `Cargo.lock` committed for reproducible module builds.
- Do not edit generated TypeScript bindings by hand.

## Domain boundaries

- Table structs (`src/repository/{domain}.rs`) hold data only: no `impl` blocks beyond derives, no methods. All behavior lives in that domain's `services` module.
- A domain's `*Services` may only touch tables defined in its own `{domain}.rs`. Every cross-domain read or write goes through the other domain's `*ReducerContext` accessor (e.g. `self.character_services().piece_exists(..)`), never a direct `self.db.<other_domain_table>()` call.
- Reducers (`reducers.rs`) only validate parameters, resolve `ctx.sender()`, and delegate to one service call; they never touch `ctx.db` themselves.
- Prefer passing `Identity` into service methods as an explicit parameter over calling `ctx.sender()` inside a service. Services may access `ctx` (validation, RNG, timestamps, UUID generation), but keep the acting identity a parameter so the same method can run against an identity other than the caller (e.g. granting to a specific player rather than only "yourself").

## Error handling

- Never write `.unwrap()`, `.expect(...)`, `panic!`, `unreachable!`, or `todo!` outside `#[cfg(test)]` code, anywhere in this workspace (`bins/*`, `sdks/*`).
- Return a typed error instead, even when the failure looks structurally impossible from local reasoning (e.g. reading back a value you just inserted, or re-deriving something already validated upstream). The invariant that makes it "safe" today can drift as the code changes; a typed error costs nothing extra to write and fails one call instead of the whole process/reducer.
- Which error type depends on where the code lives:
  - `rocknrolladb` (the SpacetimeDB module) and any sdk crate it depends on for behavior that needs one of these categories (forbidden/not-found/conflict/validation/internal) use `ServiceError`/`ServiceResult` from `sdks/rocknrolla-error`. That crate has no SpacetimeDB dependency (`ServiceError::forbidden` takes `impl Display`, not `spacetimedb::Identity`), so any sdk can depend on it without pulling in the SpacetimeDB SDK. `rocknrolladb/src/error.rs` just re-exports it so existing `crate::error::...` imports keep working.
  - A crate-local typed error (e.g. `rocknrolla-level`'s `CodecError`) stays its own enum when callers pattern-match on specific variants — don't flatten it into `ServiceError` or `anyhow::Error` just for uniformity.
  - `bins/admin` (a leaf CLI, not depended on by anything) uses `anyhow`: `bail!`/`anyhow!` for ad hoc messages, `.context()`/`.with_context()` to wrap an underlying error. Print admin errors with `{:?}` (anyhow's chained Debug), not `{}`, so wrapped context isn't silently dropped.

## Commands

- `task fmt`: run `cargo fmt`.
- `task check`: format and check all Rust targets.
- `task lint`: format and lint all targets (`cargo clippy --all-targets --all-features -- -D warnings`) — fails on any warning, anywhere in the workspace.
- `task test`: run Clippy and tests.
- `task sdk-ts`: regenerate client bindings.
- `task build`: regenerate bindings and build `rocknrolladb` for WASM.
- `task publish`: publish the development database.
- `task admin`: interactive shell to validate and import authored content (components, characters, faces, backdrops, levels, seed) from `content/`.

Run `task lint` after any Rust change, `task test` for behavior changes, and `task build` when the public module interface changes. `task lint` denies warnings workspace-wide — a warning in code you didn't touch still fails it; fix it rather than scoping around it.
