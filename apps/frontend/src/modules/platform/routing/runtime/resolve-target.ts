export interface NavigationTarget {
  pathname: string;
  search: string;
  hash: string;
}

/** `..` goes to the parent page, `members` to a child page. No trailing slash, except `/`. */
export const resolveTarget = (to: string, from: string): NavigationTarget => {
  const base = from.endsWith('/') ? from : `${from}/`;
  const url = new URL(to, `http://scylla${base}`);

  return {
    pathname: url.pathname.length > 1 ? url.pathname.replace(/\/+$/, '') : url.pathname,
    search: url.search,
    hash: url.hash,
  };
};
