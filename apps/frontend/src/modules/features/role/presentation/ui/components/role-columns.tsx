import { createColumnHelper, type ColumnDef } from '@tanstack/react-table';
import { Badge, Button } from '@shadcn';
import { ShieldCheck, Trash2, Users } from 'lucide-react';
import { Trans } from '@lingui/react/macro';
import type { RoleEntity } from '@/modules/features/role/domain/entities/role.entity.ts';
import {
  permissionName,
  scopeName,
} from '@/modules/features/role/presentation/utils/authz-labels.ts';

const columnHelper = createColumnHelper<RoleEntity>();

interface RoleColumnsMetadata {
  onViewAssignees: (role: RoleEntity) => void;
  onDelete: (id: string) => void;
}

const originLabel = (role: RoleEntity) => {
  switch (role.origin.kind) {
    case 'builtin':
      return <Trans>Built-in</Trans>;
    case 'custom':
      return <Trans>Custom</Trans>;
    default:
      return <Trans>Unknown</Trans>;
  }
};

const accessSummary = (role: RoleEntity) => {
  switch (role.access.kind) {
    case 'fullControl':
      return <Trans>Full control</Trans>;
    case 'restricted':
      return (
        <span title={role.access.permissions.map(permissionName).join(', ')}>
          <Trans>{role.access.permissions.length} permissions</Trans>
        </span>
      );
    default:
      return <Trans>Unknown</Trans>;
  }
};

export const createRoleColumns = ({
  onViewAssignees,
  onDelete,
}: RoleColumnsMetadata): ColumnDef<RoleEntity>[] => [
  columnHelper.display({
    id: 'name',
    header: () => <Trans>Name</Trans>,
    cell: info => (
      <div className='flex flex-row gap-4 items-center'>
        <div className='flex size-10 items-center justify-center rounded-lg bg-primary/10 shrink-0'>
          <ShieldCheck className='size-4 text-primary' />
        </div>
        <div className='flex flex-col items-start'>
          <p className='font-semibold text-foreground truncate'>{info.row.original.name}</p>
          <p className='font-mono text-xs text-muted-foreground truncate'>
            ID: {info.row.original.id}
          </p>
        </div>
      </div>
    ),
    size: 20,
  }),
  columnHelper.display({
    id: 'description',
    header: () => <Trans>Description</Trans>,
    cell: info => (
      <p className='text-xs text-muted-foreground'>{info.row.original.description}</p>
    ),
    size: 300,
  }),
  columnHelper.display({
    id: 'scope',
    header: () => <Trans>Scope</Trans>,
    cell: info => (
      <Badge variant='secondary'>{scopeName(info.row.original.scope)}</Badge>
    ),
    size: 100,
  }),
  columnHelper.display({
    id: 'origin',
    header: () => <Trans>Origin</Trans>,
    cell: info => (
      <Badge variant='outline'>{originLabel(info.row.original)}</Badge>
    ),
    size: 100,
  }),
  columnHelper.display({
    id: 'access',
    header: () => <Trans>Access</Trans>,
    cell: info => (
      <span className='text-sm text-muted-foreground whitespace-nowrap'>
        {accessSummary(info.row.original)}
      </span>
    ),
    size: 140,
  }),
  columnHelper.display({
    id: 'actions',
    header: () => <Trans>Actions</Trans>,
    cell: info => (
      <div className='flex items-center justify-center gap-1 shrink-0'>
        <Button
          onClick={e => {
            e.stopPropagation();
            e.preventDefault();
            onViewAssignees(info.row.original);
          }}
          type='button'
          variant='ghost'
          size='icon'
          className='size-8'
          title='View assignees'
        >
          <Users className='size-4 text-muted-foreground' />
        </Button>
        <Button
          onClick={e => {
            e.stopPropagation();
            e.preventDefault();
            onDelete(info.row.original.id);
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
  }),
];
