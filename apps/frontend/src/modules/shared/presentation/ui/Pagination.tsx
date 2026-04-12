import { Trans } from '@lingui/react/macro';
import type { PaginationInfo } from '@/modules/shared/domain/types/Pagination.ts';
import {
  Pagination as PaginationRoot,
  PaginationContent,
  PaginationEllipsis,
  PaginationItem,
  PaginationLink,
  PaginationNext,
  PaginationPrevious,
} from '@shared/presentation/ui/shadcn/pagination.tsx';

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
      <PaginationRoot>
        <PaginationContent>
          <PaginationItem>
            <PaginationPrevious
              onClick={() => hasPrevious && onPageChange(page - 1)}
              className={hasPrevious ? 'cursor-pointer' : 'pointer-events-none opacity-50'}
            />
          </PaginationItem>

          {pageItems.map((item, index) => (
            <PaginationItem key={item === 'ellipsis' ? `ellipsis-${index}` : item}>
              {item === 'ellipsis' ? (
                <PaginationEllipsis />
              ) : (
                <PaginationLink
                  isActive={item === page}
                  onClick={() => onPageChange(item)}
                  className='cursor-pointer'
                >
                  {item}
                </PaginationLink>
              )}
            </PaginationItem>
          ))}

          <PaginationItem>
            <PaginationNext
              onClick={() => hasNext && onPageChange(page + 1)}
              className={hasNext ? 'cursor-pointer' : 'pointer-events-none opacity-50'}
            />
          </PaginationItem>
        </PaginationContent>
      </PaginationRoot>

      <p className='text-sm text-muted-foreground'>
        <Trans>Showing {start}-{end} of {totalCount}</Trans>
      </p>
    </div>
  );
};
