import { Trans } from '@lingui/react/macro';
import { Badge } from '@shadcn';
import { permissionName } from '@/modules/features/role/presentation/utils/authz-labels.ts';
import type { RoleEntity } from '@/modules/features/role/domain/entities/role.entity.ts';

interface RoleDetailPermissionsProps {
  role: RoleEntity;
}

export const RoleDetailPermissions = ({ role }: RoleDetailPermissionsProps) => {
  return (
    <section className='flex flex-col gap-2'>
      <h3 className='text-xs font-semibold uppercase tracking-wider text-slate-500'>
        <Trans>Permissions</Trans>
      </h3>
      {role.access.kind === 'fullControl' ? (
        <p className='text-sm text-muted-foreground'>
          <Trans>Grants full control over its scope.</Trans>
        </p>
      ) : role.access.kind === 'restricted' ? (
        role.access.permissions.length === 0 ? (
          <p className='text-sm text-muted-foreground'>
            <Trans>No permissions.</Trans>
          </p>
        ) : (
          <div className='flex flex-wrap gap-1.5'>
            {role.access.permissions.map(permission => (
              <Badge key={permission} variant='outline' className='capitalize font-normal'>
                {permissionName(permission)}
              </Badge>
            ))}
          </div>
        )
      ) : (
        <p className='text-sm text-muted-foreground'>
          <Trans>Unknown access.</Trans>
        </p>
      )}
    </section>
  );
};

export default RoleDetailPermissions;
