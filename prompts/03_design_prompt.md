# Design RocknRolla

Nothing about the current look is final — treat this as a blank slate.
Propose a full game design: visual identity, art direction, and UI/UX for
every screen below.

## How the game works

RocknRolla is a mobile landscape physics game. The player picks a character
and rolls it downhill through an authored 2D level. There is no horizontal
steering — momentum, slopes, and collisions carry the character forward. The
player taps to jump (holding extends the jump, releasing cuts it short) and
can jump a second time in the air (double jump). The goal is to avoid hazards
and reach the finish tile; touching a lethal hazard or falling off the level
ends the run instantly, restarting it from the top with no checkpoints or
lives.

Levels are hand-authored, not procedural, and contain:

- solid ground and up/down slopes to roll over;
- non-lethal obstacles that slow, redirect, or block the player;
- lethal hazards that end the run on touch;
- water that the character floats or sinks in, depending on its buoyancy;
- fire hazards that are only survivable above a character's fire resistance;
- heavy obstacles that only dense characters can push through.

Characters are distinct data-driven "builds," not just skins: each has its
own density (sinks/floats, can/can't push heavy obstacles), jump strength,
max held-jump duration, buoyancy, and fire resistance. A run locks in one
chosen character; you can't switch mid-run.

Progression and rewards live on the backend: completing a level for the
first time unlocks its successor level(s) and grants a lootbox. Opening a
lootbox awards a random character piece; collecting every piece for a
character unlocks it. There are no purchases, ads, or social features —
just levels, characters, and collectible pieces.

## Views

1. **Boot / loading** — connecting to the backend before anything is
   playable.
2. **Level select** — browse and choose an unlocked level.
3. **Character select** — choose an unlocked character before starting a
   run (skippable if only one is unlocked).
4. **Gameplay** — the run itself, plus a minimal HUD (level/character name,
   restart, pause).
5. **Result: success** — reached the finish; leads into lootbox
   opening/reveal and piece collection.
6. **Result: defeat** — died or fell; one action back to level select, no
   retry-in-place.
7. **Collection** — characters unlocked so far and progress toward the ones
   that aren't.

## Constraints

- Mobile landscape only (16:9), must respect device safe-area insets.
- Rendered with Phaser — no HTML/CSS overlay UI.
- Large touch targets; works with touch and mouse.
- Prototype budget: this is not a production art pipeline. Assume simple,
  reproducible visuals (procedural shapes/gradients, a small consistent
  sprite/tile set, or a free/CC0 asset pack) rather than a large custom art
  commission.

## Ask

Propose a complete visual and interaction design: mood/tone, color palette,
typography, character and terrain art direction, HUD/menu style, and how
game feel (jumping, landing, hazards, rewards) should read visually. Apply it
concretely to each of the seven views above.
