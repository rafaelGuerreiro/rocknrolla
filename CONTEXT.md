# RocknRolla — Ubiquitous Language

Glossary of domain terms. Keep implementation details out; this is vocabulary only.

## Terms

### Component
A named, reusable SVG fragment of any size — from a single decor prop to a
multi-screen map section. A component carries both its **art** and its
**collider markers**; dropping a component into a level brings its physics
with it. Identified by a unique slug. "Section" is informal speech for a big
component.

### Component Library
The full set of components available for building levels. Authored as files
in the repository (the source of truth); served to the game from the
database.

### Placement
One occurrence of a component inside a level: which component, where
(x, y), at what depth (z), and how it is transformed (horizontal flip,
uniform scale). A level is composed of placements — updating a component
changes every placement of it everywhere.

### Level
A playable map: a flat list of placements plus a level-owned **spawn** point
and **finish** point, and progression metadata (name, successors, reward).
Levels do not contain art directly; all art and colliders come from
components via placements.

### Spawn / Finish
The single start and single goal position of a level. Owned by the level
itself, never embedded inside a component — so components stay freely
reusable and every level trivially has exactly one of each.

### Gameplay plane
The depth (z = 0, the center of the signed depth range) where physics
happens. Only placements on the gameplay plane produce collider bodies. All
other depths are scenery — negative z is background, positive z is
foreground.

### Parallax
The scroll-speed depth cue. Not authored directly: derived from a
placement's z by formula — far (z < 0) scrolls slower, near (z > 0)
scrolls faster, the gameplay plane scrolls 1:1.

### Collider marker
Hidden geometry inside a component's SVG (`data-t` rects/polygons) that the
client turns into physics bodies. Marker coordinates are component-local;
the placement's transform positions them in the world.

### Gallery
The dev-only HTML page that renders every component in the library for fast
visual iteration: live-reloads on file save, shows collider overlays and
dimensions. A viewer, not an editor — files are edited in the IDE.
