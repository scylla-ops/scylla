import { describe, it, expect } from 'vitest';
import { navEntriesFor, routesFor } from './compose-module-routes';
import type { ScyllaModule, ModuleRoute, NavEntry } from './scylla-module.struct.ts';
import { Permission } from '@platform/authz';
import { msg } from '@lingui/core/macro';

const moduleWith = (overrides: Partial<ScyllaModule> = {}): ScyllaModule => ({
  id: 'test-module',
  domain: {},
  ...overrides,
});

describe('navEntriesFor', () => {
  const nav = (order?: number): NavEntry => ({
    section: 'organization',
    title: msg`Entry`,
    url: 'entry',
    order,
  });

  it('flattens every module\'s nav entries', () => {
    const modules = [moduleWith({ nav: [nav(1)] }), moduleWith({ nav: [nav(2)] })];
    expect(navEntriesFor(modules)).toHaveLength(2);
  });

  it('a module with no nav contributes nothing', () => {
    expect(navEntriesFor([moduleWith(), moduleWith({ nav: [nav(1)] })])).toHaveLength(1);
  });

  it('sorts by order, ascending', () => {
    const modules = [moduleWith({ nav: [nav(3), nav(1)] }), moduleWith({ nav: [nav(2)] })];
    expect(navEntriesFor(modules).map(e => e.order)).toEqual([1, 2, 3]);
  });

  it('an entry with no order sorts as if it were 0', () => {
    const modules = [moduleWith({ nav: [nav(1), nav(undefined), nav(-1)] })];
    expect(navEntriesFor(modules).map(e => e.order)).toEqual([-1, undefined, 1]);
  });
});

describe('routesFor', () => {
  it('only includes routes declared for the requested mount', () => {
    const routes: ModuleRoute[] = [
      { mount: 'organization', path: 'a' },
      { mount: 'project', path: 'b' },
    ];
    const result = routesFor([moduleWith({ routes })], 'organization');
    expect(result.map(r => r.path)).toEqual(['a']);
  });

  it('builds a `handle` from permission/breadcrumb when either is set', () => {
    const breadcrumb = () => ({ label: msg`Users` });
    const routes: ModuleRoute[] = [
      { mount: 'organization', path: 'users', permission: Permission.LIST_USERS, breadcrumb },
    ];
    const [route] = routesFor([moduleWith({ routes })], 'organization');
    expect(route.handle).toEqual({ permission: Permission.LIST_USERS, breadcrumb });
  });

  it('omits `handle` entirely for a route with neither permission nor breadcrumb', () => {
    const routes: ModuleRoute[] = [{ mount: 'organization', path: 'public-page' }];
    const [route] = routesFor([moduleWith({ routes })], 'organization');
    expect(route).not.toHaveProperty('handle');
  });

  it('recursively converts children the same way', () => {
    const routes: ModuleRoute[] = [
      {
        mount: 'organization',
        path: 'parent',
        children: [{ mount: 'organization', path: 'child', permission: Permission.READ_PROJECT }],
      },
    ];
    const [route] = routesFor([moduleWith({ routes })], 'organization');
    expect(route.children).toHaveLength(1);
    expect(route.children?.[0]).toMatchObject({
      path: 'child',
      handle: { permission: Permission.READ_PROJECT, breadcrumb: undefined },
    });
  });

  it('preserves index/lazy/path untouched', () => {
    const lazy = () => Promise.resolve({ Component: () => null });
    const routes: ModuleRoute[] = [{ mount: 'organization', index: true, lazy }];
    const [route] = routesFor([moduleWith({ routes })], 'organization');
    expect(route.index).toBe(true);
    expect(route.lazy).toBe(lazy);
  });

  it('visits modules in registry order', () => {
    const routes = (path: string): ModuleRoute[] => [{ mount: 'organization', path }];
    const modules = [
      moduleWith({ id: 'first', routes: routes('first') }),
      moduleWith({ id: 'second', routes: routes('second') }),
    ];
    expect(routesFor(modules, 'organization').map(r => r.path)).toEqual(['first', 'second']);
  });

  describe('merging siblings that share the same path', () => {
    it('concatenates their children, in registry order', () => {
      const modules = [
        moduleWith({
          id: 'first',
          routes: [{ mount: 'organization', path: 'shared', children: [{ mount: 'organization', path: 'a' }] }],
        }),
        moduleWith({
          id: 'second',
          routes: [{ mount: 'organization', path: 'shared', children: [{ mount: 'organization', path: 'b' }] }],
        }),
      ];
      const result = routesFor(modules, 'organization');
      expect(result).toHaveLength(1);
      expect(result[0].children?.map(c => c.path)).toEqual(['a', 'b']);
    });

    it('a route with no handle at all does not clobber a sibling\'s handle', () => {
      const modules = [
        moduleWith({
          id: 'first',
          routes: [{ mount: 'organization', path: 'shared', permission: Permission.LIST_USERS }],
        }),
        moduleWith({
          id: 'second',
          routes: [{ mount: 'organization', path: 'shared', children: [{ mount: 'organization', path: 'c' }] }],
        }),
      ];
      const [route] = routesFor(modules, 'organization');
      expect(route.handle).toEqual({ permission: Permission.LIST_USERS, breadcrumb: undefined });
    });
  });
});
