export interface BreadcrumbParams {
  projectName?: string;
  organizationName?: string;
  pipelineName?: string;
}

export interface RouteHandleModel {
  breadcrumb?: string | ((params: BreadcrumbParams) => string);
}
