import { Trans } from '@lingui/react/macro';
import { ShieldCheck } from 'lucide-react';
import type { RoleEntity } from '@/modules/features/role/domain/entities/role.entity.ts';
import { RoleDetailHeader } from '@/modules/features/role/presentation/ui/components/role-detail/RoleDetailHeader.tsx';
import RoleDetailPermissions from '@/modules/features/role/presentation/ui/components/role-detail/RoleDetailPermissions.tsx';
import RoleDetailGrantList from '@/modules/features/role/presentation/ui/components/role-detail/RoleDetailGrantList.tsx';

interface RoleDetailPanelProps {
  role: RoleEntity | null;
  onEdit: (role: RoleEntity) => void;
}

export const RoleDetailPanel = ({ role, onEdit }: RoleDetailPanelProps) => {
  if (!role) {
    return (
      <div className='flex h-full flex-col items-center justify-center gap-3 text-center text-muted-foreground'>
        <div className='flex size-12 items-center justify-center rounded-xl bg-slate-100'>
          <ShieldCheck className='size-6 text-slate-400' />
        </div>
        <p className='text-sm'>
          <Trans>Select a role to see its permissions and members.</Trans>
        </p>
      </div>
    );
  }

  return (
    <div className='flex h-full flex-col gap-6 overflow-y-auto'>
      <RoleDetailHeader role={role} onEdit={onEdit} />
      <RoleDetailPermissions role={role} />
      <RoleDetailGrantList role={role} />
    </div>
  );
};
