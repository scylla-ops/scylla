import { createColumnHelper } from '@tanstack/react-table';
import { Button } from '@shadcn';
import { KeyRound, Pencil, Trash2 } from 'lucide-react';
import { Trans } from '@lingui/react/macro';
import type { Secret } from '@/modules/features/secret/domain/models/secret.model.ts';

const columnHelper = createColumnHelper<Secret>();

interface SecretColumnsMetadata {
  onDelete: (id: string) => void;
}
// eslint-disable-next-line react-refresh/only-export-components
export const createCredentialsColumns = ({ onDelete }: SecretColumnsMetadata) => [
  columnHelper.accessor('name', {
    header: () => <Trans>Name</Trans>,
    cell: info => (
      <div className={'w-fit flex flex-row '}>
        <div className={'flex flex-row gap-4 w-5/8'}>
          <div className='flex size-10 items-center justify-center rounded-lg bg-primary/10 shrink-0'>
            <KeyRound className='size-4 text-primary' />
          </div>
          <div className='flex flex-col items-start w-3/5'>
            <p className='font-semibold text-foreground truncate'>{info.row.original.name}</p>
            <p className='font-mono text-xs text-muted-foreground truncate'>
              ID: {info.row.original.id}
            </p>
          </div>
        </div>
      </div>
    ),
    size: 20,
  }),
  columnHelper.accessor('description', {
    header: 'Description',
    cell: info => (
      <div className={'w-full flex justify-center'}>
        <p className='text-xs text-muted-foreground'>{info.row.original.description}</p>
      </div>
    ),
    size: 300,
  }),
  columnHelper.accessor('createdAt', {
    header: 'Created',
    cell: info => (
      <div className={'flex justify-center'}>
        <span className='text-sm text-muted-foreground whitespace-nowrap'>
          {info.row.original.createdAt}
        </span>
      </div>
    ),
    size: 100,
  }),
  columnHelper.accessor('id', {
    header: 'Actions',
    cell: info => (
      <div className='flex items-center justify-center gap-1 shrink-0'>
        <Button type='button' variant='ghost' size='icon' className='size-8'>
          <Pencil className='size-4' />
        </Button>
        <Button
          onClick={() => onDelete(info.row.original.id)}
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
  }),
];
