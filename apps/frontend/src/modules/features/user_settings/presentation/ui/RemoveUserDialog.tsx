import React, { useState } from 'react';
import { Trans } from '@lingui/react/macro';
import {
  AlertDialog,
  AlertDialogAction,
  AlertDialogCancel,
  AlertDialogContent,
  AlertDialogDescription,
  AlertDialogHeader,
  AlertDialogTitle,
  AlertDialogTrigger,
  Button,
} from '@shadcn';

interface RemoveUserDialogProps {
  username: string;
  onConfirm: () => void;
  isLoading?: boolean;
}

export const RemoveUserDialog: React.FC<RemoveUserDialogProps> = ({
  username,
  onConfirm,
  isLoading,
}) => {
  const [open, setOpen] = useState(false);

  const handleConfirm = () => {
    onConfirm();
    setOpen(false);
  };

  return (
    <AlertDialog open={open} onOpenChange={setOpen}>
      <AlertDialogTrigger asChild>
        <Button variant='destructive' size='sm' disabled={isLoading}>
          <Trans>Remove</Trans>
        </Button>
      </AlertDialogTrigger>
      <AlertDialogContent>
        <AlertDialogHeader>
          <AlertDialogTitle>
            <Trans>Remove User</Trans>
          </AlertDialogTitle>
          <AlertDialogDescription>
            <Trans>Are you sure you want to remove {username} from the organization?</Trans>
          </AlertDialogDescription>
        </AlertDialogHeader>
        <div className='flex justify-end gap-2'>
          <AlertDialogCancel disabled={isLoading}>
            <Trans>Cancel</Trans>
          </AlertDialogCancel>
          <AlertDialogAction onClick={handleConfirm} disabled={isLoading}>
            <Trans>Remove</Trans>
          </AlertDialogAction>
        </div>
      </AlertDialogContent>
    </AlertDialog>
  );
};
