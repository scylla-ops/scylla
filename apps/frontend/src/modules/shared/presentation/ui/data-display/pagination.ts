export type PageItem = number | 'ellipsis';

/**
 * Which page numbers to show, and where the gaps go.
 *
 * Pure, and tested without a DOM: the windowing rules are the only thing in
 * `Pagination` worth pinning, and they do not need a rendered nav to be read.
 *
 * Up to seven pages every number is shown. Beyond that the window keeps the
 * first and last page reachable and slides around the current one.
 */
export const generatePageNumbers = (currentPage: number, totalPages: number): PageItem[] => {
  if (totalPages <= 7) {
    return Array.from({ length: totalPages }, (_, index) => index + 1);
  }

  if (currentPage <= 3) {
    return [1, 2, 3, 4, 'ellipsis', totalPages];
  }

  if (currentPage >= totalPages - 2) {
    return [1, 'ellipsis', totalPages - 3, totalPages - 2, totalPages - 1, totalPages];
  }

  return [1, 'ellipsis', currentPage - 1, currentPage, currentPage + 1, 'ellipsis', totalPages];
};
