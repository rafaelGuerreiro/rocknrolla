import { defineConfig } from 'vite';

export default defineConfig({
  base: './',
  server: {
    fs: {
      // The dev gallery globs repo-root content/ SVGs (see gallery.ts).
      allow: ['..'],
    },
  },
  build: {
    // Keep sprites as addressable files instead of inlined data URLs.
    assetsInlineLimit: 0,
  },
});
