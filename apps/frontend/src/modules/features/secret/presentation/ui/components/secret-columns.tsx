import type { ColumnDef } from '@tanstack/react-table';
import { Button } from '@shadcn';
import { KeyRound, Trash2 } from 'lucide-react';
import { Trans } from '@lingui/react/macro';
import { CopyableText } from '@shared/presentation/ui';
import { formatDay } from '@shared/utils/date-utils.ts';
import type { SecretEntity } from '@/modules/features/secret/domain/entities/secret.entity.ts';

interface SecretColumnsMetadata {
  onDelete: (id: string) => void;
}

export const createCredentialsColumns = ({
  onDelete,
}: SecretColumnsMetadata): ColumnDef<SecretEntity>[] => [
  {
    accessorKey: 'name',
    header: () => <Trans>Name</Trans>,
    cell: ({ row }) => (
      <div className='flex w-full min-w-0 flex-row items-center gap-4'>
        <div className='flex size-10 shrink-0 items-center justify-center rounded-lg bg-primary/10'>
          <KeyRound className='size-4 text-primary' />
        </div>
        <div className='flex min-w-0 flex-col text-start'>
          <p className='truncate font-semibold text-foreground'>{row.original.name}</p>
          <CopyableText className='text-xs text-muted-foreground/80' value={row.original.id} />
        </div>
      </div>
    ),
    size: 240,
    minSize: 240,
  },
  {
    accessorKey: 'description',
    header: () => <Trans>Description</Trans>,
    cell: ({ row }) => (
      <div className='flex w-full min-w-0 justify-center'>
        <p className='truncate text-xs text-muted-foreground'>{row.original.description}</p>
      </div>
    ),
    size: 300,
    minSize: 180,
  },
  {
    accessorKey: 'createdAt',
    header: () => <Trans>Created</Trans>,
    cell: ({ row }) => (
      <div className='flex justify-center'>
        <span className='text-sm whitespace-nowrap text-muted-foreground'>
          {formatDay(row.original.createdAt)}
        </span>
      </div>
    ),
    size: 180,
    minSize: 160,
  },
  {
    id: 'actions',
    header: () => <Trans>Actions</Trans>,
    cell: ({ row }) => (
      <div className='flex shrink-0 items-center justify-center gap-1'>
        <Button
          onClick={e => {
            e.stopPropagation();
            e.preventDefault();
            onDelete(row.original.id);
          }}
          type='button'
          variant='ghost'
          size='icon'
          className='size-8'
        >
          <Trash2 className='size-4 text-destructive' />
        </Button>
      </div>
    ),
    size: 100,
    minSize: 100,
  },
];
