/**
 * The Svelte port of the shadcn primitives.
 *
 * Ported on demand, not wholesale: a primitive lands here the phase its first
 * Svelte consumer does. `shadcn/` next door holds the React originals and is
 * **frozen** for the duration — bug fixes only — so the two cannot drift while
 * both are alive. Class strings are copied verbatim, which is why the design
 * survives the migration untouched.
 *
 * Deleted along with `shadcn/` in Phase 6.
 */
export { default as Button } from './button.svelte';
// Not from the `.svelte` file: `tsc` only ever sees a component's default
// export, so anything a `.ts` needs to import must live in a `.ts`.
export { buttonVariants, type ButtonSize, type ButtonVariant } from './button-variants.ts';

export { default as Card } from './card.svelte';
export { default as CardAction } from './card-action.svelte';
export { default as CardContent } from './card-content.svelte';
export { default as CardDescription } from './card-description.svelte';
export { default as CardFooter } from './card-footer.svelte';
export { default as CardHeader } from './card-header.svelte';
export { default as CardTitle } from './card-title.svelte';

export { default as Input } from './input.svelte';
export { default as Skeleton } from './skeleton.svelte';
