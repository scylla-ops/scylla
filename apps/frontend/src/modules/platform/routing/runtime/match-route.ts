import type { RouteParams } from '../declaration/scylla-module.struct.ts';
import type { CompiledRoute } from '../compilation/compile-routes.ts';
import { isParam, splitPath } from '../compilation/route-path.ts';

/** The route of a URL, and the values of its parameters. */
export interface RouteMatch {
  route: CompiledRoute;
  params: RouteParams;
}

const decode = (segment: string): string => {
  try {
    return decodeURIComponent(segment);
  } catch {
    return segment;
  }
};

const paramsOf = (route: CompiledRoute, segments: readonly string[]): RouteParams | null => {
  if (route.path.length !== segments.length) return null;

  const params: RouteParams = {};
  for (const [index, pattern] of route.path.entries()) {
    const segment = segments[index];
    if (isParam(pattern)) params[pattern.slice(1)] = decode(segment);
    else if (pattern !== segment) return null;
  }
  return params;
};

/**
 * The first route whose path fits `pathname`, or `null`.
 *
 * `routes` comes sorted by specificity from `compileRoutes`, so the first fit is
 * the most specific one. A trailing slash does not count.
 */
export const matchRoute = (
  routes: readonly CompiledRoute[],
  pathname: string,
): RouteMatch | null => {
  const segments = splitPath(pathname);

  for (const route of routes) {
    const params = paramsOf(route, segments);
    if (params) return { route, params };
  }
  return null;
};
