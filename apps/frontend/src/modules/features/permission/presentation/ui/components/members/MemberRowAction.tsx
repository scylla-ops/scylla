import type { ReactNode } from 'react';
import { Trans } from '@lingui/react/macro';
import { Badge } from '@shadcn';
import { Trash } from 'lucide-react';
import { IconButton } from '@shared/presentation/ui';

interface MemberRowActionProps {
  /** The signed-in user's own row: marked, never removable. */
  isCurrentUser: boolean;
  /** False when the caller may not remove this member, or there is nothing to remove. */
  canRemove: boolean;
  disabled?: boolean;
  tooltip: ReactNode;
  onRemove: () => void;
}

/**
 * The trailing control on a member row: remove them, or say that the row is you.
 *
 * Self-removal is excluded outright rather than offered and refused. It would
 * revoke the caller's own access mid-session, and the backend may reject it
 * anyway as the scope's last owner — a button whose only outcomes are "lock
 * yourself out" and "error" is not a button.
 */
export const MemberRowAction = ({
  isCurrentUser,
  canRemove,
  disabled = false,
  tooltip,
  onRemove,
}: MemberRowActionProps) => {
  if (isCurrentUser) {
    return (
      <Badge variant='outline' className='text-[10px]'>
        <Trans>You</Trans>
      </Badge>
    );
  }

  if (!canRemove) return null;

  return (
    <IconButton
      icon={Trash}
      tooltip={tooltip}
      disabled={disabled}
      onClick={onRemove}
      className='size-8 hover:bg-destructive/10 hover:text-destructive'
      iconClassName='h-3.5 w-3.5'
    />
  );
};

export default MemberRowAction;
