import { defineConfig } from 'vite';

export default defineConfig({
  base: './',
  server: {
    fs: {
      // The Kenney sprites are shared with Tiled and live at the repo root.
      allow: ['..'],
    },
  },
  build: {
    // Keep sprites as addressable files instead of inlined data URLs.
    assetsInlineLimit: 0,
  },
});
