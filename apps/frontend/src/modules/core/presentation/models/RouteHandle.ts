import type { ReactNode } from 'react';

export interface BreadcrumbParams {
  pipelineId?: string;
  projectId?: string;
}

export interface RouteHandle {
  topbar?: ReactNode;
  tabsDefaultValue?: string;
  breadcrumb?: string | ((params: BreadcrumbParams) => string);
}
