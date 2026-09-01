import type { ReactNode } from 'react';

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
 */
export interface Crumb {
  label: ReactNode;
  /** Name or id of the resource, shown in the accent colour. */
  highlight?: string;
  /** Qualifier for a sub-page, e.g. "Edit" or "Jobs". */
  detail?: ReactNode;
}

export interface RouteHandle {
  breadcrumb?: (params: BreadcrumbParams) => Crumb;
}
