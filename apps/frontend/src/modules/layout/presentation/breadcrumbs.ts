import type { MessageDescriptor } from '@lingui/core';
import type { BreadcrumbParams, TrailCrumb } from '@platform/routing';

export interface BreadcrumbItem {
  label: string;
  highlight?: string;
  detail?: string;
  pathname: string;
}

export const breadcrumbsFor = (
  trail: readonly TrailCrumb[],
  params: BreadcrumbParams,
  translate: (message: MessageDescriptor) => string,
): BreadcrumbItem[] =>
  trail.map(({ breadcrumb, pathname }) => {
    const crumb = breadcrumb(params);

    return {
      label: translate(crumb.label),
      highlight: crumb.highlight,
      detail: crumb.detail && translate(crumb.detail),
      pathname,
    };
  });
