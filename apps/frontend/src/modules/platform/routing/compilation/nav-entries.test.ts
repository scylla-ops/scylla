// @vitest-environment node
import { describe, it, expect } from 'vitest';
import { msg } from '@lingui/core/macro';
import { Permission } from '@platform/authz';
import type { NavLink, PageLoader, RouteSource } from '../declaration/scylla-module.struct.ts';
import { navEntriesFor } from './nav-entries.ts';

const page: PageLoader = () => Promise.resolve({ default: {} as never });
const nav = (order?: number): NavLink => ({ section: 'organization', title: msg`Entry`, order });
const moduleWith = (id: string, routes: RouteSource['routes']): RouteSource => ({ id, routes });

describe('navEntriesFor', () => {
  it("takes the URL and the permission of the link's route", () => {
    const [entry] = navEntriesFor([
      moduleWith('agents', {
        organization: [
          {
            path: 'agents',
            children: [{ permission: Permission.LIST_AGENTS, page, nav: nav() }],
          },
        ],
      }),
    ]);

    expect(entry).toMatchObject({ url: 'agents', permission: Permission.LIST_AGENTS });
  });

  it('sorts by order, an entry without one as 0, then by registration order', () => {
    const entries = navEntriesFor([
      moduleWith('first', {
        organization: [
          { path: 'c', page, nav: nav(3) },
          { path: 'a', page, nav: nav() },
        ],
      }),
      moduleWith('second', {
        organization: [
          { path: 'b', page, nav: nav(-1) },
          { path: 'd', page, nav: nav() },
        ],
      }),
    ]);

    expect(entries.map(entry => entry.url)).toEqual(['b', 'a', 'd', 'c']);
  });

  it('ignores the routes without a link', () => {
    expect(navEntriesFor([moduleWith('m', { organization: [{ path: 'a', page }] })])).toEqual([]);
  });

  it('refuses a link outside the organization mount, whose URL would need a project', () => {
    expect(() =>
      navEntriesFor([moduleWith('m', { project: [{ path: 'secrets', page, nav: nav() }] })]),
    ).toThrow(/Only organization routes have a sidebar link/);
  });
});
