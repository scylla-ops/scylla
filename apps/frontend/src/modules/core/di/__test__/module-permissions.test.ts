import { describe, it, expect } from 'vitest';
import { compileRoutes, navEntriesFor } from '@platform/routing';
import { appRoutes } from '@core/presentation/ui/router/core.router.ts';
import { modules } from '../registry.ts';

/**
 * Every page of the app, enumerated from the compiled routes: a new page with
 * no `permission` fails this suite. A sidebar link takes its route's permission.
 */

const guardedPages = compileRoutes(appRoutes).routes.flatMap(route =>
  route.mount !== 'public' && route.page
    ? [{ id: `/${route.path.join('/')}`, permission: route.permission }]
    : [],
);

/** A ratchet: it may shrink, never grow. Each entry says why the page has no permission. */
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
  // Enumerating nothing would pass: make sure there is something to check.
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
