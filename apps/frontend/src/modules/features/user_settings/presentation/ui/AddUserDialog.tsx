import React, { useState } from 'react';
import { Trans } from '@lingui/react/macro';
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogHeader,
  DialogTitle,
  DialogTrigger,
} from '@/modules/shared/presentation/ui/shadcn';
import { Button } from '@/modules/shared/presentation/ui/shadcn';
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@/modules/shared/presentation/ui/shadcn';

interface AddUserDialogProps {
  onAddUser: (userId: string, role: string) => void;
  isLoading?: boolean;
}

export const AddUserDialog: React.FC<AddUserDialogProps> = ({
  onAddUser: _onAddUser,
  isLoading,
}) => {
  const [open, setOpen] = useState(false);
  const [role, setRole] = useState('member');

  return (
    <Dialog open={open} onOpenChange={setOpen}>
      <DialogTrigger asChild>
        <Button>
          <Trans>Add User</Trans>
        </Button>
      </DialogTrigger>
      <DialogContent>
        <DialogHeader>
          <DialogTitle>
            <Trans>Add User to Organization</Trans>
          </DialogTitle>
          <DialogDescription>
            <Trans>Enter the user ID and select their role</Trans>
          </DialogDescription>
        </DialogHeader>
        <div className='grid gap-4 py-4'>
          <div className='grid gap-2'>
            <label htmlFor='role'>
              <Trans>Role</Trans>
            </label>
            <Select value={role} onValueChange={setRole} disabled={isLoading}>
              <SelectTrigger id='role'>
                <SelectValue />
              </SelectTrigger>
              <SelectContent>
                <SelectItem value='member'>
                  <Trans>Member</Trans>
                </SelectItem>
                <SelectItem value='admin'>
                  <Trans>Admin</Trans>
                </SelectItem>
                <SelectItem value='owner'>
                  <Trans>Owner</Trans>
                </SelectItem>
              </SelectContent>
            </Select>
          </div>
          <div className='grid gap-2'>Ici afficher liste des users</div>
        </div>
      </DialogContent>
    </Dialog>
  );
};
