export interface NavigationTarget {
  pathname: string;
  search: string;
  hash: string;
}

/**
 * Resolves a navigation target against the current pathname.
 *
 * A relative target starts from the current page, so `..` goes to the parent
 * page and `members` goes to a child page. The result never ends with a slash,
 * except for the root.
 */
export const resolveTarget = (to: string, from: string): NavigationTarget => {
  const base = from.endsWith('/') ? from : `${from}/`;
  const url = new URL(to, `http://scylla${base}`);

  return {
    pathname: url.pathname.length > 1 ? url.pathname.replace(/\/+$/, '') : url.pathname,
    search: url.search,
    hash: url.hash,
  };
};
