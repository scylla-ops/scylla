import type { MessageDescriptor } from '@lingui/core';
import type { BreadcrumbParams, TrailCrumb } from '@platform/routing';

export interface BreadcrumbItem {
  label: string;
  highlight?: string;
  detail?: string;
  pathname: string;
}

/**
 * The breadcrumbs of the current URL, from the routes that declare one.
 *
 * `label` and `detail` are translated. `highlight` is business data and stays as
 * it is.
 */
export const breadcrumbsFor = (
  trail: readonly TrailCrumb[],
  params: BreadcrumbParams,
  translate: (message: MessageDescriptor) => string,
): BreadcrumbItem[] =>
  trail.flatMap(({ handle, pathname }) => {
    if (!handle.breadcrumb) return [];
    const crumb = handle.breadcrumb(params);

    return [
      {
        label: translate(crumb.label),
        highlight: crumb.highlight,
        detail: crumb.detail && translate(crumb.detail),
        pathname,
      },
    ];
  });
