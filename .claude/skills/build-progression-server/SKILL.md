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
- A domain's `*ReducerContext` trait + `*Services<'a>` `Deref<Target = ReducerContext>` wrapper is generated, not hand-written: `extend::service_context::reducer_context!({Domain}ReducerContext, {domain}_services, {Domain}Services);` as the first item in `services.rs`, right after the `use` block. Don't write the trait/impl/struct/`Deref` boilerplate by hand — every domain's version is identical modulo these three names.

## Error handling

- Never write `.unwrap()`, `.expect(...)`, `panic!`, `unreachable!`, or `todo!` outside `#[cfg(test)]` code.
- Prefer a typed error over a panic even when a failure looks structurally impossible (e.g. reading back a row you just inserted) — the invariant can drift later, and a typed error costs nothing extra to write.
- `ServiceError` lives in `sdks/rocknrolla-error` — it has no SpacetimeDB dependency, so other sdk crates (e.g. `rocknrolla-level`) can return it too. `bins/admin` is a separate leaf CLI and uses `anyhow` instead, since it isn't depended on by anything else.
- **Never construct `ServiceError` directly inside a domain's `services.rs`, and never build a domain error's struct literal at the call site either.** Each domain that has failure conditions owns a `thiserror`-derived `<Domain>Error` enum in a sibling `errors.rs` (e.g. `repository/progression/errors.rs::ProgressionError`, `repository/lootbox/errors.rs::LootboxError`), one variant per condition with the message in its `#[error("...")]` template — never a `format!(...)` string built inline at the call site. Skip an `errors.rs` for a domain with no failure conditions (e.g. `component`) — don't create an empty enum.
  - Give the enum one `impl {Domain}Error` associated function per variant, named in snake_case after it (`UnknownLevel` → `unknown_level(level_id: Uuid) -> ServiceError`), taking the variant's fields as plain arguments (`&str` for a string field, not `impl Into<String>` — the call sites pass `&some_string_field`, which needs plain deref coercion to `&str`, not a generic bound) and returning `ServiceError` directly by building the variant and converting it inline (`{Domain}Error::Variant { .. }.into()`). Every call site then reads as one line — `LevelError::unknown_component(&import.slug, &placement.component_slug)` — with no struct literal, no `.into()`, and no enum variant name outside `errors.rs`.
  - Keep a private `impl From<{Domain}Error> for ServiceError` in the same file (used only by the constructor functions above) mapping every variant to a category (`forbidden`/`not_found`/`conflict`/`validation`/`internal`); a variant mapped to `forbidden` carries the acting `Identity` as a field since `ServiceError::forbidden` needs it.
  - Cross-cutting helpers reused by every domain (`extend/validate.rs`, `extend/access.rs`, `extend/stdb.rs`, `repository/access.rs`) are the one exception: they already are the single source of truth for their messages, so they construct `ServiceError` directly rather than each gaining a one-off enum.
  - At the call site: `return Err(SomeDomainError::variant(..));` for a direct return, and `.ok_or_else(|| SomeDomainError::variant(..))?` for an `Option` lookup — always `ok_or_else`, even for a zero-argument variant (pass the bare fn: `.ok_or_else(SomeDomainError::variant)?`), never bare `ok_or(SomeDomainError::variant(..))`. The constructor's `.into()` calls `err.to_string()` internally to build the `ServiceError` message, which always allocates, so the eager form would pay that cost even on the `Some`/`Ok` path where the error is never used.

When verification cannot run, state which manifest, module, binding target, or executable is missing.
