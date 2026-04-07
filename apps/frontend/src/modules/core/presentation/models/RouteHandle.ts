export interface BreadcrumbParams {
  pipelineId?: string;
  projectId?: string;
}

export interface RouteHandle {
  breadcrumb?: string | ((params: BreadcrumbParams) => string);
}
