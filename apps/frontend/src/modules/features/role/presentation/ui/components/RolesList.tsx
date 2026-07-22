import { useState } from 'react';
import { DataTable } from '@shared/presentation/ui/data-display/DataTable';
import { ConfirmOperationAlertDialog } from '@shared/presentation/ui/feedback/ConfirmOperationAlertDialog.tsx';
import { toast } from '@shared/presentation/utils/toast.ts';
import { useSelection } from '@shared/presentation/hooks/use-selection.ts';
import { useLingui } from '@lingui/react/macro';
import type { RoleEntity } from '@/modules/features/role/domain/entities/role.entity.ts';
import { useRoles } from '@/modules/features/role/presentation/hooks/use-roles.ts';
import { createRoleColumns } from '@/modules/features/role/presentation/ui/components/role-columns.tsx';
import { RoleAssigneesDialog } from '@/modules/features/role/presentation/ui/components/RoleAssigneesDialog.tsx';

interface RolesListProps {
  roles: RoleEntity[];
}

export const RolesList = ({ roles }: RolesListProps) => {
  const { deleteRole } = useRoles();
  const { selectedIds, select } = useSelection('roles');
  const { t } = useLingui();

  const [selectedRoleId, setSelectedRoleId] = useState<string | null>(null);
  const [assigneesRole, setAssigneesRole] = useState<RoleEntity | null>(null);

  const handleDelete = (roleId: string) => {
    deleteRole.mutate(roleId, {
      onSuccess: () => {
        toast.success(t`Role deleted`);
      },
    });
  };

  const columns = createRoleColumns({
    onViewAssignees: setAssigneesRole,
    onDelete: setSelectedRoleId,
  });

  return (
    <>
      <DataTable
        isRowSelected={row => selectedIds.includes(row.id)}
        columns={columns}
        data={roles}
        onRowClick={row => select(row.id)}
        getRowId={row => row.id}
      />
      <ConfirmOperationAlertDialog
        open={selectedRoleId !== null}
        onOpenChange={value => {
          if (!value) setSelectedRoleId(null);
        }}
        onContinue={() => {
          if (!selectedRoleId) return;
          handleDelete(selectedRoleId);
          select(selectedRoleId);
          setSelectedRoleId(null);
        }}
      />
      <RoleAssigneesDialog role={assigneesRole} onClose={() => setAssigneesRole(null)} />
    </>
  );
};
