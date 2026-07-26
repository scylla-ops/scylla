import { useMemo, useState } from 'react';
import { useFeatureSelection } from '@shared/presentation/hooks/use-feature-selection.ts';
import { Trans } from '@lingui/react/macro';
import type { RoleEntity } from '@/modules/features/role/domain/entities/role.entity.ts';
import { Permission } from '@/modules/features/role/domain/structs/permission.struct.ts';
import { useAuthorization } from '@/modules/features/role/presentation/hooks/use-authorization.ts';
import { useRoles } from '@/modules/features/role/presentation/hooks/use-roles.ts';
import { useGrants } from '@/modules/features/role/presentation/hooks/use-grants.ts';
import { RolesHeader } from '@/modules/features/role/presentation/ui/RolesHeader.tsx';
import { RoleListItem } from '@/modules/features/role/presentation/ui/components/RoleListItem.tsx';
import { RoleDetailPanel } from '@/modules/features/role/presentation/ui/components/RoleDetailPanel.tsx';
import { RoleFormDialog } from '@/modules/features/role/presentation/ui/components/RoleFormDialog.tsx';

export const RolesPage = () => {
  const { roles, deleteRole } = useRoles();
  const { grants } = useGrants();
  const { can } = useAuthorization();

  // Managing roles is an org-level capability; it gates create and bulk delete.
  const canManageRoles = can(Permission.MANAGE_ROLES);

  const [activeRoleId, setActiveRoleId] = useState<string | null>(null);
  const [formOpen, setFormOpen] = useState(false);
  const [editingRole, setEditingRole] = useState<RoleEntity | null>(null);

  const { isSelected, select, headerProps } = useFeatureSelection(
    'roles',
    useMemo(() => roles.map(role => role.id), [roles]),
    canManageRoles ? { deleteItem: (id: string) => deleteRole.mutateAsync(id) } : {},
  );

  // Members per role, derived from the grant list (no extra request).
  const memberCounts = useMemo(() => {
    const counts = new Map<string, number>();
    for (const grant of grants) {
      if (grant.target.kind === 'role') {
        counts.set(grant.target.roleId, (counts.get(grant.target.roleId) ?? 0) + 1);
      }
    }
    return counts;
  }, [grants]);

  const activeRole = roles.find(role => role.id === activeRoleId) ?? null;

  const openCreate = () => {
    setEditingRole(null);
    setFormOpen(true);
  };

  const openEdit = (role: RoleEntity) => {
    setEditingRole(role);
    setFormOpen(true);
  };

  return (
    <div className='flex flex-col gap-4 w-full h-full min-h-0'>
      <RolesHeader
        count={roles.length}
        onNew={canManageRoles ? openCreate : undefined}
        {...headerProps}
      />

      <div className='grid flex-1 min-h-0 grid-cols-1 gap-4 lg:grid-cols-[minmax(0,1.1fr)_minmax(0,1fr)]'>
        {/* Master: role list */}
        <div className='min-h-0 overflow-y-auto pr-1'>
          {roles.length === 0 ? (
            <p className='rounded-xl border border-dashed border-slate-200 py-10 text-center text-sm text-muted-foreground'>
              <Trans>No roles yet. Create one to get started.</Trans>
            </p>
          ) : (
            roles.map(role => (
              <RoleListItem
                key={role.id}
                role={role}
                memberCount={memberCounts.get(role.id) ?? 0}
                active={role.id === activeRoleId}
                selected={isSelected(role.id)}
                onOpen={() => setActiveRoleId(role.id)}
                onToggleSelect={() => {
                  select(role.id);
                }}
              />
            ))
          )}
        </div>

        {/* Detail: selected role */}
        <div className='min-h-0 rounded-xl border border-slate-200 bg-white p-5 shadow-sm'>
          <RoleDetailPanel role={activeRole} onEdit={openEdit} />
        </div>
      </div>

      <RoleFormDialog open={formOpen} role={editingRole} onClose={() => setFormOpen(false)} />
    </div>
  );
};
