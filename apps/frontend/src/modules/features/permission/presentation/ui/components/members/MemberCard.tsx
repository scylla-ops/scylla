import type { ReactNode } from 'react';
import { Plural } from '@lingui/react/macro';
import { Card } from '@shadcn';
import type {
  MemberRole,
  ScopeMember,
} from '@/modules/features/permission/domain/structs/scope-member.struct.ts';
import type { AssignableRole } from '@/modules/features/permission/presentation/hooks/use-assignable-roles.ts';
import { AddRoleSelect } from '@/modules/features/permission/presentation/ui/components/members/AddRoleSelect.tsx';
import { MemberIdentity } from '@/modules/features/permission/presentation/ui/components/members/MemberIdentity.tsx';
import { MemberRoleBadges } from '@/modules/features/permission/presentation/ui/components/members/MemberRoleBadges.tsx';
import { MemberRowAction } from '@/modules/features/permission/presentation/ui/components/members/MemberRowAction.tsx';

/**
 * What a member card needs that is the same for every card in a list. The list
 * owns it; {@link MemberCard} receives it already resolved for one member.
 */
export interface MemberCardMetadata {
  labelFor: (roleId: string) => string;
  canManage: boolean;
  disabled: boolean;
  onRevokeRole: (role: MemberRole) => void;
  /** Roles this member could still receive at the view's own scope. */
  addableRolesFor: (member: ScopeMember) => AssignableRole[];
  onAddRole: (userId: string, roleId: string) => void;
  /** Shown in the roles box when the member holds none. */
  emptyRoles?: ReactNode;
  /** False when this member has nothing removable at this scope. */
  canRemove: (member: ScopeMember) => boolean;
  removeTooltip: ReactNode;
  onRemove: (member: ScopeMember) => void;
}

interface MemberCardProps {
  name: string;
  roles: MemberRole[];
  isCurrentUser: boolean;
  canRemove: boolean;
  addableRoles: AssignableRole[];
  onAddRole: (roleId: string) => void;
  labelFor: MemberCardMetadata['labelFor'];
  canManage: boolean;
  disabled: boolean;
  onRevokeRole: (role: MemberRole) => void;
  emptyRoles?: ReactNode;
  removeTooltip: ReactNode;
  onRemove: () => void;
}

/**
 * One member: who they are, the roles they hold here, and the two operations
 * that change either.
 *
 * Three bands separated by rules — identity, roles, action — rather than one
 * padded block, because the three answer different questions and the eye should
 * not have to work out where one ends.
 *
 * The roles band has a **fixed height and scrolls**, which is the whole reason
 * the card can exist. A member's roles are a wrapping set of chips of
 * unpredictable count, so letting the band grow would give every card a
 * different height and turn the grid into a staircase. Three rows are visible,
 * the count under the name says when there are more, and every card lines up.
 */
export const MemberCard = ({
  name,
  roles,
  isCurrentUser,
  canRemove,
  addableRoles,
  onAddRole,
  labelFor,
  canManage,
  disabled,
  onRevokeRole,
  emptyRoles,
  removeTooltip,
  onRemove,
}: MemberCardProps) => (
  <Card className='gap-0 overflow-hidden py-0'>
    <div className='flex items-center gap-3 px-4 py-3.5'>
      <MemberIdentity
        name={name}
        /* The count is the scroll affordance: it is how a card carrying eight
           roles admits to it while only ever showing three rows of them. */
        subtitle={<Plural value={roles.length} _0='No role' one='# role' other='# roles' />}
      />
      <span className='ml-auto shrink-0'>
        <MemberRowAction
          isCurrentUser={isCurrentUser}
          canRemove={canRemove}
          disabled={disabled}
          tooltip={removeTooltip}
          onRemove={onRemove}
        />
      </span>
    </div>

    {/* Three chip rows tall, whatever the member holds — see the note above. */}
    <div className='h-28 overflow-y-auto border-t border-border/70 px-4 py-3'>
      <MemberRoleBadges
        roles={roles}
        labelFor={labelFor}
        canManage={canManage}
        disabled={disabled}
        onRevoke={onRevokeRole}
        empty={emptyRoles}
      />
    </div>

    {canManage && (
      // A quiet action bar, kept at a constant height: a member with every role
      // left to give and one with none must still make cards of the same size.
      <div className='flex min-h-11 items-center border-t border-border/70 bg-muted/30 px-4 py-2'>
        <AddRoleSelect disabled={disabled} roles={addableRoles} onSelect={onAddRole} />
      </div>
    )}
  </Card>
);

export default MemberCard;
