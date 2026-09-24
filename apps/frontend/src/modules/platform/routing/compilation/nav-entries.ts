import type { Permission } from '@platform/authz';
import type { NavLink, RouteSource } from '../declaration/scylla-module.struct.ts';
import { flattenModuleRoutes } from './flatten-routes.ts';

/** A sidebar entry, as the layout renders it. */
export interface NavEntry extends NavLink {
  /** The path of the page, appended to the current organization prefix. */
  url: string;
  /** The permission of the page. The link hides when the page would deny the user. */
  permission?: Permission;
}

/**
 * Every sidebar link the modules declare, sorted by `order`, then by
 * registration order.
 *
 * A link takes its URL and its permission from its route, so a link can never
 * show for a page that would deny the user.
 */
export const navEntriesFor = (modules: readonly RouteSource[]): NavEntry[] =>
  flattenModuleRoutes(modules)
    .filter(route => route.nav)
    .map(({ mount, path, nav, permission }) => {
      if (mount !== 'organization') {
        throw new Error(`Only organization routes have a sidebar link, not "${path.join('/')}".`);
      }
      return { ...(nav as NavLink), url: path.join('/'), permission };
    })
    .sort((a, b) => (a.order ?? 0) - (b.order ?? 0));
