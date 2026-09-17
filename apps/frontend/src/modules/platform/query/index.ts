/**
 * The shared query cache.
 *
 * A capability rather than a line in `core/App.tsx` because features must be
 * able to reach the client without importing the shell — and because during the
 * React → Svelte migration both bindings have to be handed the same instance.
 */
export { queryClient } from './query-client.ts';
