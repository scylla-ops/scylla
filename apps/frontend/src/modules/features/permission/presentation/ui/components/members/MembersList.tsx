import { useMemo, type ReactNode } from 'react';
import { Trans } from '@lingui/react/macro';
import { Loader2 } from 'lucide-react';
import type { ScopeMember } from '@/modules/features/permission/domain/structs/scope-member.struct.ts';
import {
  MemberCard,
  type MemberCardMetadata,
} from '@/modules/features/permission/presentation/ui/components/members/MemberCard.tsx';

interface MembersListProps extends MemberCardMetadata {
  members: ScopeMember[];
  /** Resolves a user id to a display name — falls back to the id upstream. */
  nameFor: (userId: string) => string;
  /** The signed-in user, whose own card is marked instead of removable. */
  currentUserId: string;
  isLoading?: boolean;
  /** Shown in place of the grid when nobody is listed. */
  emptyMessage?: ReactNode;
}

/** Same footprint as the grid, so the page doesn't jump between states. */
const PANEL = 'flex flex-1 items-center justify-center rounded-xl border border-dashed p-10';

/**
 * The member list, as a grid of cards, with the two states that come with it.
 *
 * Cards rather than table rows because a member is a small profile — a name and
 * a set of role chips — not a row of comparable values: the chips wrap, and in
 * a table they either stretch the row or force every column to fight for width.
 * Each card owns its own scroll instead (see {@link MemberCard}).
 *
 * Sorted by display name here rather than by the caller: the order a member
 * list comes back in is grant insertion order, which means nothing to a reader
 * scanning for a person, and both member views want the same answer.
 */
export const MembersList = ({
  members,
  nameFor,
  currentUserId,
  isLoading = false,
  emptyMessage,
  ...cardMetadata
}: MembersListProps) => {
  const sorted = useMemo(
    () =>
      [...members].sort((left, right) =>
        nameFor(left.userId).localeCompare(nameFor(right.userId)),
      ),
    [members, nameFor],
  );

  if (isLoading) {
    return (
      <div className={`${PANEL} border-border`}>
        <Loader2 className='size-5 animate-spin text-muted-foreground' />
      </div>
    );
  }

  if (members.length === 0) {
    return (
      <div className={`${PANEL} border-border`}>
        <p className='text-center text-sm text-muted-foreground'>
          {emptyMessage ?? <Trans>Nobody is listed here yet.</Trans>}
        </p>
      </div>
    );
  }

  return (
    <div className='min-h-0 flex-1 overflow-y-auto pr-1'>
      {/* Two columns until the screen is genuinely wide: a card's width is what
          decides how many role chips fit on a row, so a third column bought at
          1280px would cost every card the space the roles need. */}
      <div className='grid grid-cols-1 gap-4 lg:grid-cols-2 2xl:grid-cols-3'>
        {sorted.map(member => (
          <MemberCard
            key={member.userId}
            name={nameFor(member.userId)}
            roles={member.roles}
            isCurrentUser={member.userId === currentUserId}
            canRemove={cardMetadata.canRemove(member)}
            addableRoles={cardMetadata.addableRolesFor(member)}
            onAddRole={roleId => cardMetadata.onAddRole(member.userId, roleId)}
            onRemove={() => cardMetadata.onRemove(member)}
            labelFor={cardMetadata.labelFor}
            canManage={cardMetadata.canManage}
            disabled={cardMetadata.disabled}
            onRevokeRole={cardMetadata.onRevokeRole}
            emptyRoles={cardMetadata.emptyRoles}
            removeTooltip={cardMetadata.removeTooltip}
          />
        ))}
      </div>
    </div>
  );
};

export default MembersList;
