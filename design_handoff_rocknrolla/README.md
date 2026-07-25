# Handoff: RocknRolla — "Claymation Dusk" visual direction

## Overview
This bundle is the approved art direction and UI/UX design for RocknRolla, applied to all
seven views (boot, level select, character select, gameplay, result-success + lootbox reveal,
result-defeat, collection). The direction is **Claymation Dusk**: a warm-earthy, golden-hour
world with soft gradients, chunky rounded type, tactile round "roller" characters, a
juicy-minimal HUD, and big celebratory rewards. Tone: playful + wholesome.

**This is a reskin, not a rebuild.** The client already has every scene, the physics, the
level pipeline, the SpacetimeDB bindings, and a shared UI kit. Almost all of this work is
changing colors, fonts, shapes, and adding a few tweens/particles on top of code that already
runs. Read "What's already wired" before touching anything.

## About the design files
The files in this bundle are **design references authored in HTML/CSS/SVG** — a prototype that
shows the intended look and feel. They are **not** production code to copy. The task is to
recreate this look inside the existing Phaser/TypeScript client (`client/src`) using its
established patterns (Phaser scenes, the `ui.ts` helpers, Matter physics, the tile pipeline) —
**no HTML/CSS overlay UI**, per `client/AGENTS.md`. Everything is drawn by Phaser.

- `RocknRolla-mockups.html` — open in any browser. All seven views on one zoomable board, each
  labelled with a design note. This is the visual source of truth.
- `source/RocknRolla.dc.html` — the readable source with exact inline hex values and the six
  character **SVGs** (search for `id="char-rocco"` … `id="char-yolky"`, plus `id="flame"`).
  Lift exact values from here.

## Fidelity
**High-fidelity.** Colors, type, spacing, radii, and shadows below are final. Recreate them
faithfully with Phaser draw calls and the `ui.ts` kit. The one aspirational element is the
smooth clay slope silhouette — see "Terrain" for the pragmatic prototype approach that respects
the existing tile collision pipeline (YAGNI).

---

## Design tokens

### Palette (exact hex)
**Dusk sky gradient (top→bottom):** `#33203c` → `#6d3a4e` → `#c66240` → `#f2a63c`
**Backdrop hills:** far `#5a3550`, mid `#7a4a3f` / `#6b4a3a`
**Night/plum bases:** `#2e1c34`, `#33203c`, `#3a2440`
**Mauve mids:** `#6d3a4e`, `#864152`, `#8a4a56`, `#a8586a`
**Terracotta:** `#c66240`, `#c66a44`; dirt `#b06038`
**Amber:** `#f2a63c`, `#f2932f`; lights `#ffce7a`, `#ffe08a`, `#ffce5c`, `#ffe0a3`
**Amber CTA:** gradient `#ffce7a`→`#f2932f`, chunky bottom-shadow `#b5651f`
**Moss / ground:** surface `#9aa763`, mid `#6c7a3e`, dark `#4c5528`, accent `#6d7a44`
**Water:** `#5aa6b4`→`#2f6472`; teal accent `#3f7d8c`
**Cream surfaces (panels/HUD):** `#f5ecd8`, `#efe1c4`, `#e6d3ad`, `#e2d6bd`
**Ink (text on cream, outlines):** `#3a2a22`, `#241d16`
**Coral accent:** `#e85d3c`
**Defeat danger:** spikes `#a8503a`→`#6a2c1e`, red glow
**Star gold:** `#ffce5c`

For Phaser, these are `0x` numbers (e.g. `#c66240` → `0xc66240`). Text colors stay `'#...'`.

### Typography
Three families, loaded as web fonts (see "Font loading"):
- **Fredoka** (600 / 700) — display: wordmark, screen titles, button labels, character names,
  big numbers/stars. Replaces the current `#f5c451` Arial titles.
- **Nunito** (600–900) — UI body: stat labels, taglines, card text, counts.
- **Space Mono** (400 / 700, uppercase, letter-spacing ~0.15–0.3em) — micro labels: `CONNECTING…`,
  `READY`, `2 / 4 PIECES`, stat category caps, `MET · SPIKES`.

### Radii, shadows, motion
- Radii: buttons/pills 16–18, cards 16–22, chips 12–16, level nodes 18–22.
- **Chunky button shadow** (the signature): a solid offset block under the button
  `0 6px 0 #b5651f` **plus** soft `0 12px 22px rgba(20,10,18,.4)`. In Phaser, draw a second
  rounded-rect 6px below in the darker shade before the face.
- Cards: soft drop `0 12px 26px rgba(20,10,18,.4)`. HUD pills: `0 4px 14px rgba(20,10,18,.28)`.
- **Hero bob:** `translateY 0 → -9px`, 3s cycle, yoyo, `Sine.easeInOut` (used on the loading
  mascot, character-select hero, result hero, lootbox token).
- **Lootbox pulse:** scale `1 → 1.06`, 1.6s, yoyo. **Reveal rays:** rotate 360° over 24s linear.
- **Combo flash** ("×3 NICE!"): pop-in scale `0.6 → 1.1 → 1`, rise ~20px, fade over ~600ms.

---

## Character model — five two-layer rollers

Characters are **data-driven backend rows** (`vw_character`), rendered from a `style` string.
The mockup's stat bars map onto the **real schema fields** in
`client/src/module_bindings/vw_character_table.ts`:

| Mockup label | Backend field   | Type  | Meaning |
|--------------|-----------------|-------|---------|
| Density      | `density`       | f32   | Matter body density. ≥ `HEAVY_DENSITY` (0.0035) → can shove heavy blocks. High → sinks. |
| Jump         | `jumpSpeed`     | f32   | Jump impulse velocity. |
| Hold         | `flightTimeMs`  | u32   | Max held-jump duration (ms). |
| Buoyancy     | `buoyancy`      | f32   | ≥ ~0.5 floats in water, else sinks. |
| Fire Resist  | `fireResistance`| f32   | Survives FIRE tiles when ≥ `FIRE_RESISTANCE_THRESHOLD` (0.5). |

**The whole point of these rollers is that they are NOT round.** Irregular silhouettes make them
hard to control — flat faces catch and stall, long axes tip end-over-end, off-center weight
wobbles. This must be reflected in the **physics body**, not just the art (see "Physics bodies"
below). Design source: `source/RocknRolla-Characters.dc.html`, bundle `RocknRolla-characters.html`.

The five designed rollers, with the 5-pip bar values shown in the mockup and **suggested**
concrete field values (tune in-engine; authoritative numbers live in the server seed / import —
see `levels/seed.json` and `task server:admin`):

| Roller | `style` | Density | Jump | Hold | Buoy | Fire | handling / signature |
|--------|---------|:------:|:----:|:----:|:----:|:----:|------------|
| Rock       | `rock`  | 5 · 0.0045 | 2 · 9  | 260 | 1 · 0.15 | 3 · 0.6 | Angular & heavy — catches on edges, then lurches. **Shoves heavy blocks.** |
| Gem Shard  | `gem`   | 3 · 0.0028 | 5 · 15 | 380 | 3 · 0.55 | 4 · 0.8 | Long axis — tips end-over-end, **springs high** off a point. |
| Egg        | `egg`   | 2 · 0.0016 | 3 · 10 | 300 | 4 · 0.80 | 2 · 0.4 | Off-center weight — wobbles, self-rights. **Floats.** |
| Coconut    | `coco`  | 4 · 0.0034 | 3 · 11 | 300 | 2 · 0.35 | 3 · 0.6 | Lumpy but near-round — mostly steady, hops on its bumps. |
| Paper Ball | `paper` | 1 · 0.0009 | 2 · 8  | 240 | 5 · 0.90 | 1 · 0.15 | Featherweight — barely any momentum, bounces & drifts. |

Stat-bar rendering: 5 rounded pips per stat; filled pips use the stat's accent
(Density `#c66240`, Jump `#6d7a44`, Buoyancy `#3f7d8c`, Fire `#f2a63c`), empty pips `#e2d6bd`.

### Two-layer rig — body + face are SEPARATE (important)
Every roller is **two layers** so expressions can change at runtime and the face reads while the
body tumbles:
- **Body** (`b-rock`, `b-gem`, `b-egg`, `b-coco`, `b-paper`) — the faceless shape. Rotates with
  the physics. Carries only permanent identity marks (egg blush, coconut fibers/pores, paper
  creases), never eyes/mouth.
- **Face** (`f-happy`, `f-determined`, `f-surprised`, `f-nervous`, `f-dizzy`) — one shared set of
  eyes+mouth on a transparent field, dropped over ANY body. Stays **upright**, pinned to the
  body's centre; swap it on events.

All ten symbols are in `source/RocknRolla-Characters.dc.html` (search `id="b-rock"` …
`id="f-dizzy"`). Faces use ink `#241d16` with white glints so they read on both light (egg,
paper) and dark (rock, coconut, gem) bodies.

### Rendering strategy (SVG → Phaser textures)
Use Phaser's built-in SVG loader — no external art pipeline:

1. Add `client/src/rollers.ts` exporting **two** maps copied verbatim from the source file:
   `ROLLER_BODY_SVG: Record<string, string>` (keyed by `style`) and
   `FACE_SVG: Record<string, string>` (keyed by expression name).
2. In `BootScene.preload()`, register a texture per body and per face:
   ```ts
   for (const [style, svg] of Object.entries(ROLLER_BODY_SVG)) {
     const url = 'data:image/svg+xml;utf8,' + encodeURIComponent(svg);
     this.load.svg(`roller_${style}`, url, { width: 128, height: 128 });
   }
   for (const [name, svg] of Object.entries(FACE_SVG)) {
     const url = 'data:image/svg+xml;utf8,' + encodeURIComponent(svg);
     this.load.svg(`face_${name}`, url, { width: 72, height: 46 });
   }
   ```
3. Point `characterSpriteKey(style)` in `client/src/assets.ts` at `roller_${style}` (body texture).

### Compositing body + face in-scene
Wherever a roller is shown (gameplay, character select, collection, results), build a small
**container** of `[body, face]` rather than a lone sprite:
```ts
const body = this.add.image(0, 0, `roller_${style}`);
const face = this.add.image(0, faceOffsetY, `face_${expr}`);   // ~0.36 × body size
const roller = this.add.container(x, y, [body, face]);
// each frame (or in the body's update), keep the face level while the body spins:
body.rotation = physicsBody.angle;   // GameScene: the Matter body's angle
face.rotation = 0;                    // face never rotates
face.setTexture(`face_${currentExpr}`);
```
In **GameScene** the body is the Matter Image (its `angle` is physics-driven); add the face as a
sibling that tracks the body's `x/y` and holds `rotation = 0`. `faceOffsetY` is small and
per-body (roughly centre; nudge down a few px) — anchors match the mockup. **Swap expressions on
events:** `face_dizzy` on a lethal/hard hit, `face_surprised` on a big jump/air, `face_nervous`
near water/fire, `face_determined` as the default rolling face, `face_happy` on finish.

### Physics bodies — polygon hulls, NOT circles
The irregular shapes only play differently if the collision body matches the silhouette. Build
each body as a **Matter polygon** from its outline (`Bodies.fromVertices` / Phaser
`matter.add.fromVertices` or a `matter.bodies.fromVertices` hull), using the same vertex list as
the SVG polygon (the dashed "physics hull" drawn in the mockup is exactly this outline). A plain
circle body would roll smoothly and erase the whole design. Egg/coconut can use a smoothed hull;
rock/gem/paper want the angular vertices. Keep density from the table so heavy-push and
sink/float behavior (already implemented in `playerController`/`playerBody`) stays correct.

> If you keep the current backend `style` values (e.g. `frog`) rather than reseeding these five,
> still register the body/face textures and update the seed to reference the new `style` keys —
> a server-side change (`levels/seed.json` / `task server:admin`) that crosses the client/server
> boundary, so treat it as its own small task.

---

## Screens / views

Scene keys referenced below come from each scene's `super('...')` constructor
(`boot`, `level-select`, `character-select`, `game`, `result`, `collection` — verify in file).
Canvas is **960×540**, `Scale.FIT` + `CENTER_BOTH` (`main.ts`). Respect device safe areas by
insetting HUD ~18px and reading `capacitor` safe-area insets where available.

### 0 · Global — `client/src/ui.ts` (HIGHEST LEVERAGE)
Every menu scene draws through `button()`, `title()`, `note()`. Reskinning this one file
propagates the whole theme. Change:
- `UI_FONT` → `"Fredoka, Nunito, sans-serif"` (load fonts first).
- `button()`: face fill amber gradient `#ffce7a`→`#f2932f`, label ink `#5a2f14`, radius 16–18,
  and **draw the `0 6px 0 #b5651f` offset block** under the face. Add a secondary/cream variant
  (fill `#f5ecd8`, ink `#3a2a22`, shadow `#9a8261`) for "LEVEL SELECT"-style buttons. Disabled →
  muted cream. Keep the existing pointer/hover/tap wiring; just restyle.
- `title()` → Fredoka 700, `#f5ecd8`, subtle ink text-shadow.
- `note()` → Nunito, `#e6c9a0` on dark / `#9a7d5c` on cream.
- Also add a `pill()` helper (cream rounded pill w/ soft shadow) and a `statBars()` helper — both
  reused across HUD, level select, character select, and collection.

### Font loading
Add `@font-face`/Google Fonts `<link>` in `client/index.html`, then gate scene text on
`document.fonts.ready` (or WebFontLoader) inside `BootScene` before starting `level-select`, so
the first painted text already uses Fredoka. Phaser text does not reflow if the font loads late.

### 1 · Boot / Loading — `scenes/BootScene.ts`
- Full-screen dusk gradient (draw once with `Graphics.fillGradientStyle` to a fixed image, or a
  vertical 4-stop gradient). Two parallax hill silhouettes at the bottom (`#4a2d45`, `#6b4a3a`).
- Center: **Rocco** roller texture (~92px) bobbing above the wordmark **"RocknRolla"** (Fredoka
  700, `#f5ecd8`, ink shadow), tagline `A DOWNHILL PHYSICS ROLLER` (Space Mono, `#ffe0a3`).
- Loading bar: rounded track `rgba(30,16,26,.4)`, amber fill `#ffce7a`→`#f2932f`, a mini Rocco
  riding the fill edge; label `CONNECTING TO BASECAMP…` (Space Mono). Drive width from the real
  SpacetimeDB connect/subscription progress.

### 2 · Level Select — `scenes/LevelSelectScene.ts`
- Dusk gradient background. Levels laid out as a **dotted downhill trail** of node buttons
  (`vw_level` + `vw_my_enabled_level` / `vw_my_completed_level`).
  - Completed node: amber gradient rounded square, number in Fredoka, gold `★★★` above (fill
    stars from the level's completion/score).
  - Current/next enabled node: cream face, white border, gold halo ring (`0 0 0 5px
    rgba(255,224,138,.6)`), `PLAY` micro-label.
  - Locked node: `rgba(58,42,34,.42)` with a Fredoka `?`.
- Header pill "Choose your hill" (cream). Top-right: a collection counter pill showing a mini
  roller + `3/6` (from `vw_my_unlocked_character`), and an avatar button (current character) →
  opens `character-select` / `collection`.
- Footer micro-hint `TAP A HILL TO ROLL IN ▸`.

### 3 · Character Select — `scenes/CharacterSelectScene.ts`
- Radial spotlight backdrop (`#8a4a56`→`#3a2440`), warm glow disc behind the hero.
- **Roster rail** (left): the same roller textures at ~44px in cream chips; selected chip has an
  amber ring; locked slots show `?` (`vw_my_unlocked_character`). Large touch targets (≥52px).
- **Hero** (center): selected roller ~140px on a soft shadow, bobbing; name in Fredoka 700 below,
  a one-line tagline in Nunito.
- **Stat panel** (right): cream card, `BUILD · <NAME>` cap, four segmented stat bars (Density,
  Jump, Buoyancy, Fire Resist) using `statBars()` and the per-stat accents above.
- Back button top-left; big amber **ROLL OUT ▸** CTA bottom-center → `select_character` reducer
  then `game`. **Skip this scene** when only one character is unlocked (go straight to `game`).

### 4 · Gameplay — `scenes/GameScene.ts`
Physics, collisions, fall/finish, pause/restart, jump-puff and landing-dust are **already
implemented**. Design work:
- **Backdrop:** add a dusk sky + 2 parallax hills behind the tile grid, `setScrollFactor` low,
  `setDepth` below `GAMEPLAY_Z` (127). This is what delivers the mood over the tile terrain.
- **HUD reskin** (`buildHud()`): replace the plain name text with a cream pill (mini roller +
  `LevelName` in Fredoka + `ROLLER` in Space Mono). Restyle the ↻ / ❚❚ buttons as cream rounded
  squares via the new `ui.ts` kit; keep them `setScrollFactor(0).setDepth(300)`.
- **Juice:** recolor the existing `jumpPuff` tint to warm `#ffe0a3`; keep landing dust. Add a
  **combo flash** ("×3 NICE!", Fredoka `#ffe08a`, ink shadow, pop+rise+fade) on clean
  chained hazard clears / big air — a small tween helper, no new systems.
- **Hazard readability** (tile visuals, from `assets.ts` `TILE_SPRITES`): warm the palette so
  FIRE reads orange-gold (reuse the `#flame` SVG look via tint or a generated texture), WATER
  reads the teal `#5aa6b4`→`#2f6472`, LETHAL spikes read coral/terracotta, FINISH is the
  checkered flag. Collision labels (`lethal`,`fire`,`finish`,`heavy`) are unchanged.

### 5 · Result — Success + Lootbox reveal — `scenes/ResultScene.ts`
Two beats in one scene (the reducers `complete_level`, then `open_lootbox`, are wired):
- **Success:** warm radial backdrop, confetti burst (tween a dozen small rounded rects/circles in
  palette colors), arced **"Hill cleared!"** banner (Fredoka), three gold stars (center star
  larger), a time-vs-best chip (`TIME 0:42 · BEST 0:39`), the hero roller by a finish flag doing
  a small bob. Then a glowing, pulsing **lootbox** (wooden crate + amber glow) with a
  **TAP TO OPEN ▸** CTA (only when a lootbox was granted — first clear).
- **Lootbox reveal:** rotating radial light rays behind a **piece token** (rounded card holding
  the awarded roller's face) that flies up on a bob; broken crate lids below; labels `NEW PIECE!`
  → roller name (Fredoka); a **piece-progress row** (filled squares + dashed empties + `2 / 4
  pieces`, from `vw_my_piece` / `vw_piece`); **COLLECT ▸** CTA → `collection` or `level-select`.
  Big celebratory: confetti + rays. When a piece completes a set, escalate (extra confetti +
  "UNLOCKED!" + shake).

### 6 · Result — Defeat — `scenes/ResultScene.ts` (defeat branch)
- Dimmed dusk (dark overlay), the hazard the player hit (e.g. spikes) faintly visible.
- The roller **tumbled** (rotated) with dizzy `✦` sparkles above — playful, not harsh.
- **"Ouch!"** (Fredoka), subline "No checkpoints on this hill — back to the top you roll.", and a
  `MET · SPIKES` chip driven by the defeat reason string already produced in
  `RunOutcome`/`GameScene` (`'Wrecked by a hazard.'`, `'Burned up…'`, `'You fell out…'`).
- **One** cream **‹ LEVEL SELECT** button. No retry-in-place (per spec) — the run restarts from
  the top via level select.

### 7 · Collection — `scenes/CollectionScene.ts`
- Header: back button + **"Your Rollers"** (Fredoka) + `3 / 6 unlocked` pill.
- 3×2 grid of cards (`vw_character` × `vw_my_unlocked_character` / `vw_my_piece`):
  - **Unlocked:** cream card, full-color roller (~58px), name, a compact 5-dot signature-stat row,
    a green `READY` tag, and the trait caption (`DENSE · SINKS`, `FIREPROOF`, `SHOVES HEAVY`…).
  - **In-progress/locked:** dark dashed card, **silhouetted** roller (tint black + low alpha),
    name, a **piece-progress row** (filled/dashed squares), and `n / 4 PIECES` in amber. This
    progress is the collection pull.

---

## Interactions & behavior
- All input is Phaser **pointer** (mouse + touch share one path) — keep it that way.
- Gameplay: tap = jump; hold extends (up to `flightTimeMs`); release cuts short; second tap in
  air = double jump. Already handled by `PlayerController` — do not reimplement.
- Navigation flow: `boot → level-select → (character-select) → game → result → level-select`,
  with `result` branching into the lootbox reveal on first clears, and `collection` reachable
  from level select / character select.
- Tween durations/easings are in "Design tokens → motion". Prefer `Sine`/`Back.easeOut` for the
  bouncy, wholesome feel; avoid linear except the reveal rays.

## What's already wired (do NOT rebuild)
- Matter physics, gravity, world bounds, camera follow (`gameplay/*`).
- Player body + controller (jump/hold/double-jump/buoyancy/fire), density-based heavy-push.
- Level decode + tile build (`levels.ts`, `levelBuilder.ts`), RLE, content hashing.
- Collision→outcome (`runOutcome.ts`, `GameScene.bindCollisions`), fall/finish detection.
- Jump-puff + landing-dust particles (`textures.ts`, `GameScene`).
- SpacetimeDB tables + reducers: `select_character`, `complete_level`, `open_lootbox`, piece/
  lootbox/character/level views (`module_bindings/`). Do not hand-edit generated bindings.
- Scene registration + 960×540 FIT scaling (`main.ts`).

Your job is **presentation**: `ui.ts`, per-scene visuals, fonts, the roller textures, the dusk
backdrop, and a few celebratory tweens.

## Terrain — the one tradeoff
The mockup shows smooth clay slopes. The engine's terrain is **tile-based** (`CELL = 64`, TILE
ids → Kenney sprites) and collisions depend on it. Reshaping terrain into smooth polygons would
mean rewriting the level/collision pipeline — out of scope for a prototype (YAGNI, per
`AGENTS.md`). **Recommended:** keep tile collisions exactly as-is; deliver the mood with (1) the
painted dusk **backdrop + parallax hills** behind the grid, and (2) warm **recoloring/tinting**
of the Kenney tiles (or swapping to the pack's warmer dirt/sand variants). This gets ~90% of the
look for ~10% of the cost. Revisit smooth slopes only if the prototype graduates.

## Files to touch (real paths)
- `client/index.html` — font `<link>`s.
- `client/src/ui.ts` — theme the button/title/note kit; add `pill()`, `statBars()`.
- `client/src/rollers.ts` **(new)** — `ROLLER_BODY_SVG` (5 bodies) + `FACE_SVG` (5 faces) maps.
- `client/src/assets.ts` — `characterSpriteKey` → `roller_${style}` (body texture).
- `client/src/scenes/BootScene.ts` — body+face SVG texture preload, font gate, boot visuals.
- a small roller container helper (body+face, face pinned upright) reused by the scenes.
- `client/src/gameplay/playerBody.ts` — build the Matter body as a polygon hull, not a circle.
- `client/src/scenes/LevelSelectScene.ts`, `CharacterSelectScene.ts`, `CollectionScene.ts` —
  reskin per specs above.
- `client/src/scenes/GameScene.ts` — backdrop, HUD reskin, warm particle tint, combo flash.
- `client/src/scenes/ResultScene.ts` — success + lootbox reveal + defeat branches.
- `client/src/textures.ts` — optional warm hazard/backdrop textures.
- (server) `levels/seed.json` / `task server:admin` — only if reseeding the five rollers.

## Verify (from `client/AGENTS.md`)
Run `task fmt`, `task lint`, `task build` after client changes, and play the affected scenes in
a browser (`task dev`). If reseeding characters, use `task server:admin` and the server checks.

## How to view the design
- `RocknRolla-mockups.html` — the seven views (layout / UX / colour source of truth). Pull exact
  hex from `source/RocknRolla.dc.html`.
- `RocknRolla-characters.html` — the **five two-layer rollers**: faceless bodies, swappable
  faces, a live body-rolls-face-stays demo, tumble strips, and the expression library. Pull the
  ten SVG symbols (`b-*`, `f-*`) from `source/RocknRolla-Characters.dc.html`. This file is the
  authoritative roller art; the seven-view mockups use an earlier round placeholder roller for
  layout only — render the five rollers above in-engine.
