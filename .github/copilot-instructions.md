# RocknRolla

Treat the root `AGENTS.md` as the source of truth for repository structure, architecture, scope, and verification.

- Keep this prototype simple and apply YAGNI.
- Keep Phaser/Matter gameplay in `client/`, progression in `server/`, and packaging automation in `build/`.
- The client is authoritative for physics and reports waypoints; the server intentionally trusts those reports.
- Do not add server-side physics, anti-cheat, speculative abstractions, or dependencies unless explicitly requested.
- Use the authenticated SpacetimeDB sender for ownership even though gameplay results are trusted.
- Make the smallest working change and run only the relevant available checks.
