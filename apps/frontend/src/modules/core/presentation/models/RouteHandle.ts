export interface BreadcrumbParams {
  projectName?: string;
  organizationName?: string;
  pipelineName?: string;
}

export interface RouteHandle {
  breadcrumb?: string | ((params: BreadcrumbParams) => string);
}
