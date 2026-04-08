import { Button } from '@shadcn';
import { ChevronLeft, ChevronRight } from 'lucide-react';
import type { PaginationInfo } from '@/modules/shared/domain/types/Pagination.ts';

interface PaginationProps {
  paginationInfo: PaginationInfo;
  onPageChange: (page: number) => void;
  className?: string;
}

type PageItem = number | 'ellipsis';

const generatePageNumbers = (currentPage: number, totalPages: number): PageItem[] => {
  if (totalPages <= 7) {
    return Array.from({ length: totalPages }, (_, i) => i + 1);
  }

  if (currentPage <= 3) {
    return [1, 2, 3, 4, 'ellipsis', totalPages];
  }

  if (currentPage >= totalPages - 2) {
    return [1, 'ellipsis', totalPages - 3, totalPages - 2, totalPages - 1, totalPages];
  }

  return [1, 'ellipsis', currentPage - 1, currentPage, currentPage + 1, 'ellipsis', totalPages];
};

export const Pagination = ({ paginationInfo, onPageChange, className }: PaginationProps) => {
  const { page, pageSize, totalCount, totalPages, hasNext, hasPrevious } = paginationInfo;
  const pageItems = generatePageNumbers(page, totalPages);

  const start = (page - 1) * pageSize + 1;
  const end = Math.min(page * pageSize, totalCount);

  return (
    <div className={`flex flex-col items-center gap-2 ${className ?? ''}`}>
      <nav aria-label="Pagination" className="flex items-center gap-1">
        <Button
          variant="outline"
          size="icon"
          className="min-h-11 min-w-11"
          disabled={!hasPrevious}
          onClick={() => onPageChange(page - 1)}
          aria-label="Previous page"
        >
          <ChevronLeft className="size-4" />
        </Button>

        {pageItems.map((item, index) =>
          item === 'ellipsis' ? (
            <span
              key={`ellipsis-${index}`}
              className="flex min-h-11 min-w-11 items-center justify-center text-muted-foreground"
              aria-hidden
            >
              ...
            </span>
          ) : (
            <Button
              key={item}
              variant={item === page ? 'default' : 'outline'}
              size="icon"
              className="min-h-11 min-w-11"
              onClick={() => onPageChange(item)}
              aria-label={`Page ${item}`}
              aria-current={item === page ? 'page' : undefined}
            >
              {item}
            </Button>
          ),
        )}

        <Button
          variant="outline"
          size="icon"
          className="min-h-11 min-w-11"
          disabled={!hasNext}
          onClick={() => onPageChange(page + 1)}
          aria-label="Next page"
        >
          <ChevronRight className="size-4" />
        </Button>
      </nav>

      <p className="text-sm text-muted-foreground">
        Showing {start}-{end} of {totalCount}
      </p>
    </div>
  );
};
