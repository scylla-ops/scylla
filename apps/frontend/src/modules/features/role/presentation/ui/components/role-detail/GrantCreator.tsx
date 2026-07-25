import type { PermissionScope } from '@/modules/features/role/domain/structs/permission.struct.ts';

interface GrantCreatorProps {
  scope: PermissionScope;
}
/** Component used to create a grant
 * If the scope of the role is system, it select an user to apply the grant
 * If the scope of the role is organization, it select an user to apply the grant and an organization to what the grant offer the access
 * If the scope of the role is project, it select an user, an organization and a project inside this organization
 * **/
export const GrantCreator = ({ scope }: GrantCreatorProps) => {
  return <div>Grant creation</div>;
};

export default GrantCreator;
