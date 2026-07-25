import { Pencil, ShieldCheck } from 'lucide-react';
import { Trans } from '@lingui/react/macro';
import { Badge, Button } from '@shadcn';
import type { RoleEntity } from '@/modules/features/role/domain/entities/role.entity.ts';
import { scopeName } from '@/modules/features/role/presentation/utils/authz-labels.ts';

interface RoleDetailHeaderProps {
  role: RoleEntity;
  onEdit: (role: RoleEntity) => void;
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

export const RoleDetailHeader = ({ role, onEdit }: RoleDetailHeaderProps) => {
  return (
    <section className={'flex flex-col gap-4'}>
      <div className='flex items-start justify-between gap-4'>
        <div className='flex items-start gap-3 min-w-0'>
          <div className='flex size-11 items-center justify-center rounded-xl bg-primary/10 shrink-0'>
            <ShieldCheck className='size-5 text-primary' />
          </div>
          <div className='min-w-0'>
            <h2 className='text-xl font-bold tracking-tight truncate'>{role.name}</h2>
            <p className='text-sm text-muted-foreground'>
              {role.description || <Trans>No description</Trans>}
            </p>
          </div>
        </div>
        <Button
          variant='outline'
          disabled={role.origin.kind === 'builtin'}
          onClick={() => onEdit(role)}
        >
          <Pencil className='size-4' />
          <Trans>Edit</Trans>
        </Button>
      </div>
      <div className='flex flex-wrap gap-2'>
        <Badge variant='secondary'>{scopeName(role.scope)}</Badge>
        <Badge variant='outline'>{originLabel(role)}</Badge>
        {role.access.kind === 'fullControl' && (
          <Badge>
            <Trans>Full control</Trans>
          </Badge>
        )}
      </div>
    </section>
  );
};

export default RoleDetailHeader;
