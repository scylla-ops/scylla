import { scyllaNavigate, type ScyllaNavigate } from './scylla-navigate.ts';

/**
 * The React binding over `scyllaNavigate`.
 *
 * It used to hold the URL building itself, on top of `useNavigate` and
 * `useLocation`. Those moved into `scylla-navigate.ts`, which has no framework
 * in it, so a Svelte view model can navigate the same way a React component
 * does — and there is one implementation of the URLs rather than two.
 *
 * Still a hook, and still the shape callers expect (`const { goToJobs } =
 * useScyllaNavigate()`), so nothing in the React tree changed. Phase 6 deletes
 * this file and its callers import `scyllaNavigate` directly.
 */
export const useScyllaNavigate = (): ScyllaNavigate => scyllaNavigate;
