# Claude Code kickoff prompt

Paste this into Claude Code from the repo root (`rocknrolla/`). It sequences the reskin
lowest-risk-first so the game stays playable at every step.

---

We're applying an approved visual direction ("Claymation Dusk") to the RocknRolla client. The
full spec, design tokens, per-scene breakdown, and character data are in
`design_handoff_rocknrolla/README.md`. Visual sources of truth:
`design_handoff_rocknrolla/RocknRolla-mockups.html` (the seven views) and
`design_handoff_rocknrolla/RocknRolla-characters.html` (the five two-layer rollers). Exact hex
values and every SVG symbol are in `design_handoff_rocknrolla/source/*.dc.html`. Read the README
first.

This is a **reskin over working logic** — physics, level pipeline, collisions, reducers, and all
six scenes already exist. Do NOT rebuild gameplay systems or hand-edit `src/module_bindings/`.
Follow `AGENTS.md` and `client/AGENTS.md`: Phaser-only (no HTML/CSS overlay UI), YAGNI, keep
changes inside the component you're editing. The rollers are irregular on purpose — build their
Matter bodies as polygon hulls, never circles. After each step run `task fmt`, `task lint`,
`task build`, and play the affected scene with `task dev`.

Do it in this order, checking in after each:

1. **Fonts + theme kit.** Load Fredoka / Nunito / Space Mono in `client/index.html`, gate first
   text on `document.fonts.ready` in `BootScene`. Retheme `client/src/ui.ts` to the token
   palette (amber chunky buttons with the `0 6px 0 #b5651f` offset block, cream variant, Fredoka
   titles) and add `pill()` + `statBars()` helpers. This alone reskins every menu.

2. **Roller textures (two-layer rig).** Add `client/src/rollers.ts` with `ROLLER_BODY_SVG` (the
   five faceless bodies `b-rock/gem/egg/coco/paper`) and `FACE_SVG` (the five expressions
   `f-happy/determined/surprised/nervous/dizzy`), copied from
   `source/RocknRolla-Characters.dc.html`. Preload each via `this.load.svg(...)` in `BootScene`
   (`roller_<style>`, `face_<expr>`), point `characterSpriteKey` at `roller_${style}`, and add a
   small container helper that stacks a face over a body with `face.rotation = 0` so the face
   stays upright while the body spins. Build the player's Matter body as a **polygon hull** from
   the shape's vertices (`playerBody.ts`), NOT a circle — the irregular shape is the gameplay.

3. **BootScene** visuals — dusk gradient, parallax hills, bobbing Rocco, wordmark, loading bar
   driven by real connect/subscription progress.

4. **LevelSelectScene** — dotted downhill trail of node buttons (completed / current-halo /
   locked), header pill, collection counter, footer hint.

5. **CharacterSelectScene** — spotlit bobbing hero, roster rail, cream stat panel with
   `statBars()`, ROLL OUT CTA; skip the scene when only one character is unlocked.

6. **GameScene** — dusk backdrop + parallax behind the tile grid (depth < 127), HUD reskin
   (cream pill + restyled ↻/❚❚), warm jump-puff tint, a combo-flash tween, warm hazard palette.

7. **ResultScene** — success beat (confetti, "Hill cleared!", stars, time-vs-best, pulsing
   lootbox → TAP TO OPEN), lootbox-reveal beat (rotating rays, flying piece token, piece
   progress, COLLECT), and the defeat branch ("Ouch!", tumbled roller, MET · <hazard>, single
   LEVEL SELECT button).

8. **CollectionScene** — 3×2 grid: unlocked cards (full-color roller + signature stats + READY +
   trait), locked cards (silhouette + `n / 4 PIECES`).

Terrain stays tile-based — deliver its mood via the painted backdrop and warm tile tinting only;
do not rewrite the level/collision pipeline (see README "Terrain — the one tradeoff").

Optionally, once the client looks right, propose (separately) reseeding the five rollers into
`levels/seed.json` via `task server:admin`, since that crosses into the server.
