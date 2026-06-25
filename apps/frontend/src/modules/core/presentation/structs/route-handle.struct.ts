export interface BreadcrumbParams {
  projectName?: string;
  organizationName?: string;
  pipelineName?: string;
  userId?: string;
}

export interface RouteHandle {
  breadcrumb?: string | ((params: BreadcrumbParams) => string);
}
