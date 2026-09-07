import { Outlet, useMatches } from 'react-router-dom';
import { RequirePermission } from '@platform/authz';
import type { RouteHandle } from './route-handle.struct.ts';

/**
 * Applies the `permission` a route declared in its handle.
 *
 * One pathless layout route per mount point replaces wrapping every element in
 * `<RequirePermission>` by hand — which was 15 near-identical wrappers that had
 * to be kept in step with the sidebar's own permission list.
 *
 * The deepest match wins: a child route asking for more than its parent gets
 * checked against its own requirement.
 */
export const RouteGuard = () => {
  const matches = useMatches();

  const required = matches
    .map(match => (match.handle as RouteHandle | undefined)?.permission)
    .filter(permission => permission !== undefined)
    .at(-1);

  if (required === undefined) return <Outlet />;

  return (
    <RequirePermission permission={required}>
      <Outlet />
    </RequirePermission>
  );
};
