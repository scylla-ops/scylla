import { vitePreprocess } from '@sveltejs/vite-plugin-svelte';

/**
 * Svelte 5, runes only.
 *
 * `runes: true` is not the default yet — a component without a single rune is
 * otherwise compiled in legacy mode, which silently gives it the old reactivity
 * and lets `export let` through. Forcing it here means the compiler rejects the
 * legacy syntax instead of accepting a mix of two models across the codebase.
 *
 * No SvelteKit: `apps/frontend/dist` is embedded in the Rust binary by
 * `rust-embed`, so there is never a Node runtime to host server routes.
 * See `refacto_svelte.md` §0.
 *
 * @type {import('@sveltejs/vite-plugin-svelte').SvelteConfig}
 */
export default {
  preprocess: vitePreprocess(),
  compilerOptions: { runes: true },
};
