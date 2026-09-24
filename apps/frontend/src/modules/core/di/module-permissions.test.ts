import { describe, it, expect } from 'vitest';
import { compileRoutes, navEntriesFor } from '@platform/routing';
import { appRoutes } from '@core/presentation/ui/router/core.router.ts';
import { modules } from './registry.ts';

/**
 * Conformance over every route in the app — not a behaviour test of any one of
 * them.
 *
 * A per-component test pins a gate someone already wrote; it cannot fail for a
 * gate someone forgot. This rule enumerates the compiled routes, so a feature
 * added tomorrow is checked the day it joins the registry and the default is
 * fail-closed: a new page with no `permission` turns the suite red without
 * anyone remembering to write a test for it.
 *
 * A sidebar link needs no check of its own: it takes the permission of its route.
 */

const guardedPages = compileRoutes(appRoutes).routes.flatMap(route =>
  route.mount !== 'public' && route.page
    ? [{ id: `/${route.path.join('/')}`, permission: route.permission }]
    : [],
);

/**
 * Pages deliberately reachable without a declared permission.
 *
 * A ratchet, like the coverage thresholds: this list may shrink, never grow.
 * Each entry states why the page cannot simply declare one — "we haven't got to
 * it yet" is not a reason, it is a missing `permission`.
 */
const UNGATED_PAGES: Readonly<Record<string, string>> = {
  '/':
    'The landing of the shell: it only sends the user on to the dashboard of an organization, ' +
    'which declares its own permission.',
  '/:organizationSlug/users/:userId':
    'Doubles as "my own profile": the layout sends every user to /users/me. Gating it on ' +
    'LIST_USERS would lock a user out of their own settings, so the page needs to tell self ' +
    'from other before it can carry a permission.',
  '/:organizationSlug/marketplace':
    'TRIAGE: the page reads a hardcoded catalog — DefaultMarketplaceRepository calls no backend, ' +
    'so there is nothing to deny yet, and the enum has no marketplace permission to declare. ' +
    'Gate it when the real data layer lands.',
};

describe('module permission declarations', () => {
  // A test that enumerates can pass by enumerating nothing. If `compileRoutes`
  // or the registry ever stops yielding pages here, that is the bug — not a green run.
  it('finds the registry pages it is supposed to check', () => {
    expect(guardedPages.length).toBeGreaterThan(10);
    expect(navEntriesFor(modules).length).toBeGreaterThan(0);
  });

  describe('every page behind the auth guard declares a permission', () => {
    it.each(guardedPages.map(page => [page.id, page] as const))('%s', (id, page) => {
      if (id in UNGATED_PAGES) {
        expect(
          page.permission,
          `${id} now declares a permission — drop it from UNGATED_PAGES, the list only shrinks.`,
        ).toBeUndefined();
        return;
      }

      expect(
        page.permission,
        `${id} renders a page that anyone logged in can reach. Declare a \`permission\` on the ` +
          'route in its `*.module.ts` — the guard and the sidebar both read it. If the page ' +
          'genuinely needs none, add it to UNGATED_PAGES with the reason.',
      ).toBeDefined();
    });
  });

  it('the ungated list names only routes that still exist', () => {
    const known = new Set(guardedPages.map(page => page.id));
    const stale = Object.keys(UNGATED_PAGES).filter(id => !known.has(id));

    expect(stale, 'these UNGATED_PAGES entries match no route — delete them').toEqual([]);
  });
});
