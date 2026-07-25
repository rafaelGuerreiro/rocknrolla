# 05 — Characters and backdrops live in SpacetimeDB

**Status: implemented 2026-07-07** (dev DB republished via
`task unsafe-overwrite`; verified headless in-browser).

Move character art and level backdrops into SpacetimeDB using the same
workflow as components: authored files in the repo (source of truth) →
admin import → private tables → public views → client builds textures from
DB bytes. Decided in a grilling session on 2026-07-07; glossary updated
(`CONTEXT.md`: Backdrop, Character, Face).

## Decisions (locked)

1. **"Scene" = Backdrop** — a named per-level scenery theme (sky + far
   strip + mid strip). Canonical term is **Backdrop** (avoids Phaser Scene
   collision; matches `buildBackdrop` vocabulary).
2. **Character art scope**: bodies + faces + silhouettes all move to DB.
3. **Silhouettes are derived at import** by the admin importer using the
   existing filter-injection trick (`rollers.ts` `SILHOUETTE_FILTER`), not
   authored. Importer validates each body SVG keeps `<defs>…</defs>` so
   the injection stays valid.
4. **Repo restructure**: `levels/` → `content/`:
   - `content/seed.json` (was `levels/seed.json`)
   - `content/levels/*.json` (was `levels/src/`)
   - `content/components/*.svg` (was `levels/components/`)
   - `content/characters/<style>.svg` (new — ported `ROLLER_BODY_SVG`)
   - `content/faces/<name>.svg` (new — ported `FACE_SVG`)
   - `content/backdrops/<slug>.{sky,far,mid}.svg` (new)
   Update Taskfile paths, admin defaults, gallery glob, READMEs.
5. **Character art schema**: separate table with FK; faces separate:
   - `character_art_v1 { id, character_id (btree), kind: body|silhouette,
     width_px, height_px, content_hash, data }`
   - `face_v1 { id, slug unique, width_px, height_px, content_hash, data }`
6. **`style` becomes import-time only**: stays in `seed.json` +
   `character_def_v1` as the filename↔seed link; **dropped from
   `vw_character_v1`**. Client resolves art by `character_id`, textures
   keyed by content hash (like `component_<hash>`).
7. **Backdrop authoring**: three files per backdrop —
   `<slug>.sky.svg`, `<slug>.far.svg`, `<slug>.mid.svg`. Slug from prefix,
   role from suffix; each file is a standalone SVG parsed with the existing
   component file rules (root integer `width`/`height`).
8. **Backdrop schema**: one row, fixed columns —
   `backdrop_v1 { id, slug unique, sky_(w,h,hash,data),
   far_(w,h,hash,data), mid_(w,h,hash,data) }`. Three-layer contract is
   schema-enforced. Join-free view.
9. **Level link**: every level JSON gains required `"backdrop": "<slug>"`
   (all 18 existing levels: `"dusk"`). Import fails on unresolved slug —
   no silent default. `level_v1` gains `backdrop_id: Uuid`; exposed on
   `vw_level_v1`. Import order: seed → components → character art → faces
   → backdrops → levels.
10. **Everything waits for DB**: BootScene shows a bare solid color +
    progress until the subscription lands (no SVG preloads, no pre-connect
    mascot). After connect, all screens use DB art. Menus use a client
    constant `DEFAULT_BACKDROP_SLUG = 'dusk'`.
11. **Gallery shows all content**: sections for components (with collider
    overlay, unchanged), characters (body + derived silhouette), faces,
    and backdrops (three layers composed). Same HMR loop.

## Asserted defaults (flag if wrong)

- Spotlight and rays stay procedural client UI chrome (`textures.ts`
  keeps `ensureParticleTextures`, spotlight, rays; loses sky/hills).
- `vw_backdrop_v1` exposes all backdrops (tiny table; menus need the
  default one even if no level references it).
- Dusk backdrop SVGs are bootstrapped once (port of the procedural sky
  gradient + ellipse-bump hills), committed files become source of truth;
  no permanent svggen painters for the new content types.
- Client drops the unknown-style→rock fallback: missing art fails loud.
- Boot's bouncing mascot is dropped (nothing to draw pre-connect).
- Parallax factors and layer placement stay feel knobs in `tuning.ts`.

## Implementation outline

### Content files
- Port the five bodies from `client/src/rollers.ts` to
  `content/characters/<style>.svg` — add root `width="120" height="120"`,
  keep `<defs>`.
- Port the five faces to `content/faces/<name>.svg` — root
  `width="80" height="50"`.
- Author `content/backdrops/dusk.{sky,far,mid}.svg` matching today's
  procedural look (`textures.ts` SKY_STOPS, hillTexture colors/bumps).
- Add `"backdrop": "dusk"` to all 18 level JSONs.

### Server (`server/`)
- New tables + import reducers + views mirroring the component pattern:
  `character_art_v1`, `face_v1`, `backdrop_v1` (module-owner-gated
  `import_*_v1` reducers, shared SVG validation in
  `sdks/rocknrolla-level`).
- `level_v1` += `backdrop_id`; level import validates it resolves.
- `vw_character_v1` drops `style`.
- Admin: load characters/faces/backdrops dirs (componentsrc-style
  parsers), derive silhouettes (filter injection + `<defs>` check),
  resolve backdrop slugs → uuids, extend `import all` order, update
  default paths for `content/`.
- Republish via `task unsafe-overwrite` (pre-live; wipes + reseeds).

### Client (`client/`)
- Regenerate bindings (`task build`).
- `rollers.ts`: delete `ROLLER_BODY_SVG`, `FACE_SVG`, silhouette
  derivation; keep composition helpers (`addRoller`, face ratios).
  Textures created from view rows, hash-keyed.
- `textures.ts`: delete sky/hill generation.
- `BootScene`: solid-color loading; build textures after subscription.
- `GameScene`: backdrop from the level's `backdrop_id` (sky stretched
  image, far/mid tileSprites, current parallax handling).
- Menu scenes: backdrop via `DEFAULT_BACKDROP_SLUG`.
- `db.ts`: subscribe to the new views.
- Gallery: glob `content/{components,characters,faces,backdrops}`,
  render new sections.

### Tests
- Admin parser tests (characters/faces/backdrops, silhouette derivation,
  missing-art and missing-`<defs>` failures).
- Module validation tests (backdrop slug resolution, layer contract).
- Client `levels.test.ts`-style unit tests for new mapping helpers.
- `task server:lint`, `task server:test`, `task client:fmt`,
  `task client:lint`, `task client:build`, `task client:test`; then
  in-browser check of boot → menu → a level with the dusk backdrop.
