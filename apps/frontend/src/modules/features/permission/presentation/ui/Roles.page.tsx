import { useRoles } from '@/modules/features/permission/presentation/hooks/use-roles.ts';
import { RolesHeader } from '@/modules/features/permission/presentation/ui/RolesHeader.tsx';

export const RolesPage = () => {
  const { roles } = useRoles();

  return (
    <div>
      <RolesHeader count={roles.length} />
      Table
    </div>
  );
};
