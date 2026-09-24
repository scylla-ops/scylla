import type { MessageDescriptor } from '@lingui/core';

export interface BreadcrumbParams {
  projectName?: string;
  organizationName?: string;
  pipelineName?: string;
  userId?: string;
  jobId?: string;
}

/**
 * `label` and `detail` are translated; `highlight` is data, shown as it is.
 * Descriptors, so that routes can be declared in a plain `.ts` file.
 */
export interface Crumb {
  label: MessageDescriptor;
  highlight?: string;
  /** E.g. "Edit" or "Jobs". */
  detail?: MessageDescriptor;
}

export type BreadcrumbFn = (params: BreadcrumbParams) => Crumb;
