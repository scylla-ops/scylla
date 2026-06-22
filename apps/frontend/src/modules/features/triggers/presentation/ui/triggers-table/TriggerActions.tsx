import type { SyntheticEvent } from 'react';
import { EditIcon, Loader2, PlayIcon, Trash2 } from 'lucide-react';
import { Trans } from '@lingui/react/macro';
import { IconButton } from '@shared/presentation/ui';

interface TriggerActionsProps {
  onFire: (e: SyntheticEvent) => void;
  onEdit: (e: SyntheticEvent) => void;
  onDelete: (e: SyntheticEvent) => void;
  isFiring?: boolean;
}

/** Row actions: fire now (test), edit, delete. */
export const TriggerActions = ({ onFire, onEdit, onDelete, isFiring }: TriggerActionsProps) => (
  <div className='flex items-center justify-center gap-1'>
    <IconButton
      icon={isFiring ? Loader2 : PlayIcon}
      tooltip={<Trans>Fire now</Trans>}
      onClick={onFire}
      iconClassName={isFiring ? 'animate-spin' : 'fill-current'}
    />
    <IconButton icon={EditIcon} tooltip={<Trans>Edit</Trans>} onClick={onEdit} />
    <IconButton
      icon={Trash2}
      tooltip={<Trans>Delete</Trans>}
      onClick={onDelete}
      iconClassName='text-destructive'
    />
  </div>
);
