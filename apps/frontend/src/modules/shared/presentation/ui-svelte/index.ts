/**
 * The Svelte half of `shared/presentation/ui/`, mirroring it group for group.
 *
 * A parallel tree rather than `.svelte` files dropped beside their `.tsx`
 * originals: the two versions of a component share a name, so one barrel cannot
 * export both. `ui/` is frozen for the duration of the migration — bug fixes
 * only — and in Phase 6 it is deleted and this folder takes its place, which is
 * a rename rather than a merge.
 *
 * The vendored primitives are the exception: they already live at
 * `ui/shadcn-svelte/`, beside `ui/shadcn/`, and keep their own `@shadcn-svelte`
 * alias.
 */
export * from './controls';
export * from './data-display';
export * from './editor/code-mirror.actions.ts';
export * from './feedback';
export * from './forms';
export * from './layout';
export * from './motion/page-transition.ts';
export * from './motion/reduced-motion.ts';
export type { LucideIcon } from './icon.ts';
