import { msg } from '@lingui/core/macro';

export const paginationMessages = {
  /** Placeholder names are part of the msgid — keep `start`, `end`, `totalCount`. */
  showing: (start: number, end: number, totalCount: number) =>
    msg`Showing ${start}-${end} of ${totalCount}`,
};
