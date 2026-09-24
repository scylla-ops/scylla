/**
 * A route path as a list of segments: `/:organizationSlug/agents` is
 * `[':organizationSlug', 'agents']`. A segment that starts with `:` is a parameter.
 */
export type RoutePath = readonly string[];

/** `'agents/:agentId'` → `['agents', ':agentId']`. Empty segments are dropped. */
export const splitPath = (path = ''): string[] => path.split('/').filter(Boolean);

/** `['agents', ':agentId']` → `'/agents/:agentId'`. */
export const joinPath = (path: RoutePath): string => `/${path.join('/')}`;

export const isParam = (segment: string): boolean => segment.startsWith(':');

/**
 * The identity of a path. Parameter names do not count: `/users/:id` and
 * `/users/:userId` are the same URLs, so they are the same route.
 */
export const pathKey = (path: RoutePath): string =>
  joinPath(path.map(segment => (isParam(segment) ? ':' : segment)));

/** True when `path` starts with every segment of `prefix`. A path is its own prefix. */
export const startsWithPath = (path: RoutePath, prefix: RoutePath): boolean =>
  prefix.length <= path.length && pathKey(path.slice(0, prefix.length)) === pathKey(prefix);

/**
 * Orders the more specific path first: at the first segment where the two
 * differ in kind, a static segment beats a parameter. So `/login` is tried
 * before `/:organizationSlug`.
 */
export const bySpecificity = (a: RoutePath, b: RoutePath): number => {
  for (let index = 0; index < Math.min(a.length, b.length); index++) {
    const rank = Number(isParam(a[index])) - Number(isParam(b[index]));
    if (rank !== 0) return rank;
  }
  return 0;
};
