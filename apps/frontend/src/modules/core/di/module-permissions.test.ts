// @vitest-environment node
import { describe, it, expect } from 'vitest';
import type { RouteObject } from 'react-router-dom';
import { navEntriesFor, routesFor } from '@platform/routing';
import type { RouteHandle, RouteMount } from '@platform/routing';
import { Permission } from '@platform/authz';
import { modules } from './registry.ts';

/**
 * Conformance over every module declaration in the registry — not a behaviour
 * test of any one of them.
 *
 * A per-component test pins a gate someone already wrote; it cannot fail for a
 * gate someone forgot. These rules enumerate from `modules`, so a feature added
 * tomorrow is checked the day it joins the registry and the default is
 * fail-closed: a new page with no `permission` turns the suite red without
 * anyone remembering to write a test for it.
 *
 * They run on the *composed* trees (`routesFor`) rather than the raw
 * declarations. That is what the router actually mounts — cross-module merges
 * included, such as `user` owning `users` while `organization` owns
 * `users/:userId` — and it is where a leaf's effective permission is decided.
 */

interface Landing {
  readonly permission?: Permission;
}

interface PageRoute extends Landing {
  /** `mount/path`, which is how a failure names the offender. */
  readonly id: string;
}

/** Mirrors `RouteGuard`: the deepest permission declared along the chain wins. */
const effectivePermission = (route: RouteObject, inherited?: Permission): Permission | undefined =>
  (route.handle as RouteHandle | undefined)?.permission ?? inherited;

/** Readable in a failure message, where a bare enum value is just a number. */
const nameOf = (permission?: Permission): string =>
  permission === undefined ? 'no permission' : Permission[permission];

const segmentOf = (route: RouteObject): string => route.path ?? (route.index ? '(index)' : '');

/**
 * Every route that renders something, with the permission that guards it.
 *
 * A route without `lazy` renders an `Outlet`: it exists only to own a path
 * segment and its children carry the gate, so it is walked through rather than
 * reported.
 */
const collectPages = (
  routes: readonly RouteObject[],
  prefix: string,
  inherited: Permission | undefined,
): PageRoute[] =>
  routes.flatMap(route => {
    const permission = effectivePermission(route, inherited);
    const id = [prefix, segmentOf(route)].filter(Boolean).join('/');

    return [
      ...(route.lazy ? [{ id, permission }] : []),
      ...collectPages(route.children ?? [], id, permission),
    ];
  });

/** The mounts that sit behind `AuthGuard`. `public` is outside it by definition. */
const GUARDED_MOUNTS = [
  'organization',
  'projects',
  'project',
] as const satisfies readonly RouteMount[];

const guardedPages = GUARDED_MOUNTS.flatMap(mount =>
  collectPages(routesFor(modules, mount), mount, undefined),
);

/**
 * Pages deliberately reachable without a declared permission.
 *
 * A ratchet, like the coverage thresholds: this list may shrink, never grow.
 * Each entry states why the page cannot simply declare one — "we haven't got to
 * it yet" is not a reason, it is a missing `permission`.
 */
const UNGATED_PAGES: Readonly<Record<string, string>> = {
  'organization/users/:userId':
    'Doubles as "my own profile": the layout sends every user to /users/me. Gating it on ' +
    'LIST_USERS would lock a user out of their own settings, so the page needs to tell self ' +
    'from other before it can carry a permission.',
  'organization/marketplace':
    'TRIAGE: the page reads a hardcoded catalog — DefaultMarketplaceRepository calls no backend, ' +
    'so there is nothing to deny yet, and the enum has no marketplace permission to declare. ' +
    'Gate it when the real data layer lands.',
};

/**
 * The segments the shell owns under `/:organizationSlug`, and the mount whose
 * routes fill them. A nav `url` is appended to that prefix, so this is what
 * turns a sidebar link into the route it lands on. Adding a mount to
 * `Core.router.tsx` means adding it here.
 */
const SHELL_SEGMENTS: Readonly<Record<string, RouteMount>> = { projects: 'projects' };

/** What react-router renders for a URL: follow index children, deepest gate wins. */
const descendToLanding = (route: RouteObject, inherited?: Permission): Landing | undefined => {
  const permission = effectivePermission(route, inherited);
  const index = (route.children ?? []).find(child => child.index);

  if (index) return descendToLanding(index, permission);
  return route.lazy ? { permission } : undefined;
};

const navLanding = (url: string): Landing | undefined => {
  const shellMount = SHELL_SEGMENTS[url];
  // The shell owns the segment, so stand in for it: a parent with no gate of
  // its own whose children are that mount's routes.
  if (shellMount) return descendToLanding({ children: routesFor(modules, shellMount) });

  const route = routesFor(modules, 'organization').find(candidate => candidate.path === url);
  return route && descendToLanding(route);
};

describe('module permission declarations', () => {
  // A test that enumerates can pass by enumerating nothing. If `routesFor` or
  // the registry ever stops yielding pages here, that is the bug — not a green run.
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

  describe('a sidebar link and the page it opens require the same permission', () => {
    it.each(navEntriesFor(modules).map(entry => [entry.url, entry] as const))(
      '%s',
      (url, entry) => {
        const landing = navLanding(url);

        expect(
          landing,
          `the "${url}" sidebar entry points at /:organizationSlug/${url}, which no module ` +
            'route renders.',
        ).toBeDefined();

        // Compared by name so a mismatch reads as LIST_USERS vs MANAGE_ROLES
        // rather than 5 vs 41.
        expect(
          nameOf(entry.permission),
          `the "${url}" sidebar entry is shown to holders of ${nameOf(entry.permission)}, but ` +
            `the page it opens requires ${nameOf(landing?.permission)}. One of the two ` +
            'declarations is wrong: a visible link must never lead to a denial.',
        ).toBe(nameOf(landing?.permission));
      },
    );
  });
});
