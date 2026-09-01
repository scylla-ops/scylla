import { Button } from '@shadcn';
import { ChevronLeft, ChevronRight } from 'lucide-react';
import { Trans } from '@lingui/react/macro';

interface CredentialsPaginationProps {
  page: number;
  totalPages: number;
  totalItems: number;
  itemsPerPage: number;
  onPageChange: (page: number) => void;
}

export const SecretPagination = ({
  page,
  totalPages,
  totalItems,
  itemsPerPage,
  onPageChange,
}: CredentialsPaginationProps) => {
  const firstItem = totalItems === 0 ? 0 : (page - 1) * itemsPerPage + 1;
  const lastItem = Math.min(page * itemsPerPage, totalItems);

  return (
    <div className='rounded-xl border border-border/60 bg-card px-4 py-3 flex items-center justify-between gap-4'>
      <p className='text-sm text-muted-foreground'>
        <Trans>
          Showing <span className='font-semibold'>{firstItem}</span>-
          <span className='font-semibold'>{lastItem}</span> of{' '}
          <span className='font-semibold'>{totalItems}</span> credentials
        </Trans>
      </p>
      <div className='flex items-center gap-2'>
        <Button
          variant='outline'
          size='sm'
          disabled={page <= 1}
          onClick={() => onPageChange(page - 1)}
          className='h-8 gap-1'
        >
          <ChevronLeft className='size-4' />
          <span className='hidden sm:inline'>
            <Trans>Prev</Trans>
          </span>
        </Button>
        <span className='text-sm text-muted-foreground min-w-fit'>
          {page} / {totalPages}
        </span>
        <Button
          variant='outline'
          size='sm'
          disabled={page >= totalPages}
          onClick={() => onPageChange(page + 1)}
          className='h-8 gap-1'
        >
          <span className='hidden sm:inline'>
            <Trans>Next</Trans>
          </span>
          <ChevronRight className='size-4' />
        </Button>
      </div>
    </div>
  );
};
