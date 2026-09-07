import type { MessageDescriptor } from '@lingui/core';
import type { Permission } from '@platform/authz';

export interface BreadcrumbParams {
  projectName?: string;
  organizationName?: string;
  pipelineName?: string;
  userId?: string;
}

/**
 * A crumb keeps its translatable words apart from the business identifier it
 * points at: `label` and `detail` are rendered through Lingui, `highlight` is
 * data and stays verbatim in every locale.
 *
 * They are `msg` descriptors rather than JSX so that routes — and therefore the
 * breadcrumbs — can be declared in a module's plain `.ts` DI file. They are
 * still translated at render time, so a locale switch updates them.
 */
export interface Crumb {
  label: MessageDescriptor;
  /** Name or id of the resource, shown in the accent colour. */
  highlight?: string;
  /** Qualifier for a sub-page, e.g. "Edit" or "Jobs". */
  detail?: MessageDescriptor;
}

export interface RouteHandle {
  breadcrumb?: (params: BreadcrumbParams) => Crumb;
  /** Read by `RouteGuard`; set from a module's `ModuleRoute.permission`. */
  permission?: Permission;
}
