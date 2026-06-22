export interface BreadcrumbParams {
  projectName?: string;
  organizationName?: string;
  pipelineName?: string;
  userId?: string;
}

export interface RouteHandleModel {
  breadcrumb?: string | ((params: BreadcrumbParams) => string);
}
