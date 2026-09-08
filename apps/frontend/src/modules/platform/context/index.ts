/**
 * Which organization / project / pipeline the user is currently looking at, and
 * navigation derived from it.
 *
 * This is business context, not generic plumbing, which is why it sits in
 * `platform/` rather than `shared/` — but it is deliberately identifier-only
 * (ids and display names), so it never needs to know what a project *is*.
 */
export { useContextStore } from './use-context.store.ts';
export { useScyllaNavigate } from './use-scylla-navigate.ts';
