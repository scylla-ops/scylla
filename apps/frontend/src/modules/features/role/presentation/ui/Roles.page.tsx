import { useRoles } from '@/modules/features/role/presentation/hooks/use-roles.ts';
import { RolesHeader } from '@/modules/features/role/presentation/ui/RolesHeader.tsx';
import { RolesList } from '@/modules/features/role/presentation/ui/components/RolesList.tsx';

export const RolesPage = () => {
  const { roles } = useRoles();

  return (
    <div className='flex flex-col gap-4 w-full min-h-full'>
      <RolesHeader count={roles.length} />
      <div className='overflow-hidden'>
        <RolesList roles={roles} />
      </div>
    </div>
  );
};
