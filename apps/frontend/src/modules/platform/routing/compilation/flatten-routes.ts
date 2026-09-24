import type { ModuleRoute, RouteMount, RouteSource } from '../declaration/scylla-module.struct.ts';
import { joinPath, pathKey, splitPath, type RoutePath } from './route-path.ts';

/** What a route declares for its path, without its children. */
export type RouteFields = Omit<ModuleRoute, 'path' | 'children'>;

/** One route of a module, unfolded: its full path from the mount, and no children. */
export interface FlatRoute extends RouteFields {
  mount: RouteMount;
  /** From the mount, e.g. `['agents', ':agentId']`. */
  path: RoutePath;
  /** The modules that declared it, to name them in an error. */
  declaredBy: readonly string[];
}

const FIELDS = ['page', 'redirect', 'permission', 'breadcrumb', 'nav'] as const;

/**
 * Unfolds a tree of routes: each route gets the path of its parents in front
 * of its own. A parent comes before its children.
 */
export const unfoldRoutes = (
  routes: readonly ModuleRoute[],
  origin: { mount: RouteMount; moduleId: string },
  parentPath: RoutePath = [],
): FlatRoute[] =>
  routes.flatMap(({ path, children = [], ...fields }) => {
    const fullPath = [...parentPath, ...splitPath(path)];

    return [
      { ...fields, mount: origin.mount, path: fullPath, declaredBy: [origin.moduleId] },
      ...unfoldRoutes(children, origin, fullPath),
    ];
  });

const describe = (route: FlatRoute): string => `${route.mount} route "${joinPath(route.path)}"`;

const mergeTwo = (into: FlatRoute, route: FlatRoute): FlatRoute => {
  const merged: FlatRoute = { ...into, declaredBy: [...into.declaredBy, ...route.declaredBy] };

  for (const field of FIELDS) {
    if (route[field] === undefined) continue;
    if (into[field] !== undefined) {
      throw new Error(
        `${describe(into)} declares \`${field}\` twice (${merged.declaredBy.join(', ')}).`,
      );
    }
    Object.assign(merged, { [field]: route[field] });
  }

  return merged;
};

const assertCoherent = (route: FlatRoute): void => {
  if (route.page && route.redirect) {
    throw new Error(`${describe(route)} has both a \`page\` and a \`redirect\`.`);
  }
  if ((route.permission !== undefined || route.nav) && !route.page) {
    throw new Error(`${describe(route)} has a \`permission\` or a \`nav\` but no \`page\`.`);
  }
};

/**
 * Makes one route of the routes that share a mount and a path.
 *
 * This is how two modules own parts of one path without importing each other:
 * one declares the crumb of `users`, the other the page of `users/:userId`, and a
 * third may add the page of `users`. Each field of a path is declared once: a
 * second `page`, `permission`, … on the same path is an error, not an override.
 * Parameter names do not count, so `:id` and `:userId` are the same path.
 */
export const mergeSamePath = (routes: readonly FlatRoute[]): FlatRoute[] => {
  const byPath = new Map<string, FlatRoute>();

  for (const route of routes) {
    const key = `${route.mount}${pathKey(route.path)}`;
    const existing = byPath.get(key);
    byPath.set(key, existing ? mergeTwo(existing, route) : route);
  }

  const merged = [...byPath.values()];
  merged.forEach(assertCoherent);
  return merged;
};

/**
 * Every route the modules declare, one per mount and path, in registration order.
 *
 * The two steps of the flattening: `unfoldRoutes` turns each module's tree into
 * a list, `mergeSamePath` joins what several declarations say about one path.
 */
export const flattenModuleRoutes = (modules: readonly RouteSource[]): FlatRoute[] =>
  mergeSamePath(
    modules.flatMap(({ id, routes = {} }) =>
      (Object.keys(routes) as RouteMount[]).flatMap(mount =>
        unfoldRoutes(routes[mount] ?? [], { mount, moduleId: id }),
      ),
    ),
  );
