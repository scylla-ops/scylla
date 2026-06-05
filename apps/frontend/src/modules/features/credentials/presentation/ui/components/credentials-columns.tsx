import { createColumnHelper } from '@tanstack/react-table';
import type {
  Credential,
  CredentialKind,
} from '@/modules/features/credentials/domain/models/credential.model.ts';
import { Badge, Button } from '@shadcn';
import {
  AlertTriangle,
  CheckCircle2,
  Clock3,
  KeyRound,
  Pencil,
  RotateCcw,
  Trash2,
} from 'lucide-react';
import { Trans } from '@lingui/react/macro';

const KIND_LABELS: Record<CredentialKind, string> = {
  SSH_KEY: 'SSH KEY',
  TOKEN: 'TOKEN',
  SECRET_TEXT: 'SECRET TEXT',
  LOGIN: 'LOGIN',
};

const getKindBadgeClass = (kind: CredentialKind) => {
  if (kind === 'SSH_KEY') return 'bg-blue-100 text-blue-700 border-blue-200';
  if (kind === 'TOKEN') return 'bg-emerald-100 text-emerald-700 border-emerald-200';
  if (kind === 'SECRET_TEXT') return 'bg-purple-100 text-purple-700 border-purple-200';
  return 'bg-amber-100 text-amber-700 border-amber-200';
};

export const HealthBadge = ({ credential }: { credential: Credential }) => {
  if (credential.health === 'warning') {
    return (
      <div className='w-full flex items-center gap-2 text-xs'>
        <AlertTriangle className='size-3.5 text-red-500' />
        <span className='text-red-500'>Expires in {credential.expiresInDays ?? 0} days</span>
      </div>
    );
  }

  if (credential.health === 'healthy') {
    return (
      <div className='w-full flex items-center gap-2 text-xs'>
        <CheckCircle2 className='size-3.5 text-emerald-500' />
        <span className='text-emerald-500'>Healthy</span>
      </div>
    );
  }

  return (
    <div className='w-full flex items-center gap-2 text-xs'>
      <Clock3 className='size-3.5 text-muted-foreground' />
      <span className='text-muted-foreground'>No usage in 30 days</span>
    </div>
  );
};

const columnHelper = createColumnHelper<Credential>();

// eslint-disable-next-line react-refresh/only-export-components
export const createCredentialsColumns = () => [
  columnHelper.accessor('name', {
    header: () => <Trans>Name</Trans>,
    cell: info => (
      <div className={'w-full flex flex-row '}>
        <div className={'flex flex-row gap-4 w-5/8'}>
          <div className='flex size-10 items-center justify-center rounded-lg bg-primary/10 shrink-0'>
            <KeyRound className='size-4 text-primary' />
          </div>
          <div className='flex flex-col items-start w-3/5'>
            <p className='font-semibold text-foreground truncate'>{info.row.original.name}</p>
            <p className='font-mono text-xs text-muted-foreground truncate'>
              ID: {info.row.original.externalId}
            </p>
          </div>
        </div>
      </div>
    ),
    size: 120,
  }),
  columnHelper.accessor('kind', {
    header: 'Kind',
    cell: info => (
      <div className={'flex justify-center'}>
        <Badge variant='outline' className={getKindBadgeClass(info.row.original.kind)}>
          {KIND_LABELS[info.row.original.kind]}
        </Badge>
      </div>
    ),
    size: 120,
  }),
  columnHelper.accessor('health', {
    header: 'Health / Usage',
    cell: info => (
      <div className={'w-full flex justify-center'}>
        <div className='w-9/11 overflow-visible flex flex-col'>
          <div className={'ml-8 w-full flex flex-col'}>
            <HealthBadge credential={info.row.original} />
            <p className='text-xs text-muted-foreground'>{info.row.original.lastUsageLabel}</p>
          </div>
        </div>
      </div>
    ),
    size: 50,
  }),
  columnHelper.accessor('createdAtLabel', {
    header: 'Created',
    cell: info => (
      <div className={'flex justify-center'}>
        <span className='text-sm text-muted-foreground whitespace-nowrap'>
          {info.row.original.createdAtLabel}
        </span>
      </div>
    ),
    size: 140,
  }),
  columnHelper.accessor('id', {
    header: 'Actions',
    cell: () => (
      <div className='flex items-center justify-center gap-1 shrink-0'>
        <Button type='button' variant='ghost' size='icon' className='size-8'>
          <RotateCcw className='size-4' />
        </Button>
        <Button type='button' variant='ghost' size='icon' className='size-8'>
          <Pencil className='size-4' />
        </Button>
        <Button type='button' variant='ghost' size='icon' className='size-8'>
          <Trash2 className='size-4 text-destructive' />
        </Button>
      </div>
    ),
    size: 120,
  }),
];
