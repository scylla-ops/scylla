// @vitest-environment node
import { describe, it, expect } from 'vitest';
import type { Component } from 'svelte';
import { Permission } from '@platform/authz';
import type {
  AppRouterConfig,
  LayoutComponent,
  RouteWrapper,
} from '../../declaration/app-router-config.struct.ts';
import type { ModuleRoutes, PageLoader } from '../../declaration/scylla-module.struct.ts';
import { compileRoutes, type CompiledRoute } from '../compile-routes.ts';
import { flattenModuleRoutes } from '../flatten-routes.ts';
import { joinPath } from '../route-path.ts';

const page: PageLoader = () => Promise.resolve({ default: {} as never });
const crumb = (label: string) => () => ({ label: { id: label, message: label } });
const Layout = {} as LayoutComponent;
const OrgWrapper = {} as RouteWrapper;
const ProjectWrapper = {} as RouteWrapper;

const configWith = (...routes: ModuleRoutes[]): AppRouterConfig => ({
  mounts: {
    public: {},
    app: { layout: Layout },
    organization: { parent: 'app', path: ':organizationSlug', wrapper: OrgWrapper },
    project: {
      parent: 'organization',
      path: 'projects/:projectId',
      wrapper: ProjectWrapper,
      breadcrumb: crumb('Project'),
    },
  },
  modules: routes.map((declared, index) => ({ id: `module-${index}`, routes: declared })),
  fallback: {} as Component,
});

const compile = (...routes: ModuleRoutes[]) => compileRoutes(configWith(...routes)).routes;
const paths = (...routes: ModuleRoutes[]) => compile(...routes).map(route => joinPath(route.path));
const routeAt = (path: string, ...routes: ModuleRoutes[]) =>
  compile(...routes).find(route => joinPath(route.path) === path);
const labelsOf = (route?: CompiledRoute) =>
  route?.trail.map(mark => mark.breadcrumb({}).label.message);

describe('flattenModuleRoutes', () => {
  it('gives each child the path of its parents', () => {
    const flat = flattenModuleRoutes([
      {
        id: 'agents',
        routes: {
          organization: [{ path: 'agents', page, children: [{ path: ':agentId/logs', page }] }],
        },
      },
    ]);

    expect(flat.map(route => joinPath(route.path))).toEqual(['/agents', '/agents/:agentId/logs']);
  });

  it("makes a child without a path its parent's own page", () => {
    const flat = flattenModuleRoutes([
      {
        id: 'agents',
        routes: {
          organization: [{ path: 'agents', breadcrumb: crumb('Agents'), children: [{ page }] }],
        },
      },
    ]);

    expect(flat).toHaveLength(1);
    expect(flat[0]).toMatchObject({ path: ['agents'], page });
    expect(flat[0].breadcrumb).toBeDefined();
  });

  it('merges what two modules declare on one path, whatever they name the parameters', () => {
    const flat = flattenModuleRoutes([
      { id: 'user', routes: { organization: [{ path: 'users/:id', breadcrumb: crumb('User') }] } },
      { id: 'organization', routes: { organization: [{ path: 'users/:userId', page }] } },
    ]);

    expect(flat).toHaveLength(1);
    expect(flat[0].declaredBy).toEqual(['user', 'organization']);
  });

  it('keeps apart the same path under two mounts', () => {
    const flat = flattenModuleRoutes([
      {
        id: 'membership',
        routes: { organization: [{ path: 'members', page }], project: [{ path: 'members', page }] },
      },
    ]);

    expect(flat).toHaveLength(2);
  });

  it('refuses a field that two declarations of one path both set', () => {
    const declare = () =>
      flattenModuleRoutes([
        { id: 'first', routes: { organization: [{ path: 'users', page }] } },
        { id: 'second', routes: { organization: [{ path: 'users', page }] } },
      ]);

    expect(declare).toThrow(/"\/users" declares `page` twice \(first, second\)/);
  });

  it('refuses a route with both a page and a redirect', () => {
    expect(() =>
      flattenModuleRoutes([
        { id: 'm', routes: { organization: [{ path: 'a', page, redirect: 'b' }] } },
      ]),
    ).toThrow(/both a `page` and a `redirect`/);
  });

  it('refuses a permission without a page to guard', () => {
    expect(() =>
      flattenModuleRoutes([
        { id: 'm', routes: { organization: [{ path: 'a', permission: Permission.LIST_USERS }] } },
      ]),
    ).toThrow(/`permission` or a `nav` but no `page`/);
  });
});

describe('compileRoutes', () => {
  it('places each route under the path of its mount', () => {
    expect(
      paths({
        public: [{ path: 'login', page }],
        organization: [{ path: 'agents', page }],
        project: [{ path: 'secrets', page }],
      }),
    ).toEqual([
      '/login',
      '/:organizationSlug/agents',
      '/:organizationSlug/projects/:projectId/secrets',
    ]);
  });

  it('keeps only the routes that render something: a page or a redirect', () => {
    expect(
      paths({
        organization: [{ redirect: 'dashboard' }, { path: 'users', breadcrumb: crumb('Users') }],
      }),
    ).toEqual(['/:organizationSlug']);
  });

  it('puts a static segment before a parameter, so /login is tried before /:organizationSlug', () => {
    expect(paths({ organization: [{ page }] }, { public: [{ path: 'login', page }] })).toEqual([
      '/login',
      '/:organizationSlug',
    ]);
  });

  it('gives a route the layout of its root mount and the wrappers of its mounts, outermost first', () => {
    const route = routeAt('/:organizationSlug/projects/:projectId/secrets', {
      project: [{ path: 'secrets', page }],
    });

    expect(route?.layout).toBe(Layout);
    expect(route?.wrappers).toEqual([OrgWrapper, ProjectWrapper]);
  });

  it('gives a public route no layout and no wrapper', () => {
    const route = routeAt('/login', { public: [{ path: 'login', page }] });

    expect(route?.layout).toBeUndefined();
    expect(route?.wrappers).toEqual([]);
  });

  it("keeps a route's own permission, and never lends it to a child", () => {
    const routes = {
      organization: [
        {
          path: 'users',
          permission: Permission.LIST_USERS,
          page,
          children: [{ path: ':userId', page }],
        },
      ],
    };

    expect(routeAt('/:organizationSlug/users', routes)?.permission).toBe(Permission.LIST_USERS);
    expect(routeAt('/:organizationSlug/users/:userId', routes)?.permission).toBeUndefined();
  });

  describe('the trail of a page', () => {
    it('holds the crumb of every path it starts with, from the root, with the depth of each', () => {
      const route = routeAt(
        '/:organizationSlug/projects/:projectId/pipelines/:pipelineId/jobs/:jobId',
        { project: [{ path: 'pipelines/:pipelineId/jobs', breadcrumb: crumb('Jobs') }] },
        {
          project: [{ path: 'pipelines/:pipelineId/jobs/:jobId', breadcrumb: crumb('Job'), page }],
        },
        { organization: [{ path: 'projects', breadcrumb: crumb('Projects'), page }] },
      );

      expect(labelsOf(route)).toEqual(['Projects', 'Project', 'Jobs', 'Job']);
      expect(route?.trail.map(mark => mark.depth)).toEqual([2, 3, 6, 7]);
    });

    it('skips the crumbs of paths the page is not under', () => {
      const route = routeAt('/:organizationSlug/agents', {
        organization: [
          { path: 'agents', page },
          { path: 'users', breadcrumb: crumb('Users') },
        ],
      });

      expect(route?.trail).toEqual([]);
    });
  });

  it('refuses two pages that two mounts put on one URL', () => {
    expect(() =>
      compile({ organization: [{ path: 'projects/:id', page }], project: [{ page }] }),
    ).toThrow(/Two mounts declare a page on "\/:organizationSlug\/projects\/:id"/);
  });
});
