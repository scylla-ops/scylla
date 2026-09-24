import { msg } from '@lingui/core/macro';

export const paginationMessages = {
  /** Keep the placeholder names: they are part of the msgid. */
  showing: (start: number, end: number, totalCount: number) =>
    msg`Showing ${start}-${end} of ${totalCount}`,
};
