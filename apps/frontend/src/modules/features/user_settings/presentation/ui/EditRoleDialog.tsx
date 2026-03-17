import React, { useState } from 'react';
import { Trans } from '@lingui/react/macro';
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogHeader,
  DialogTitle,
  DialogTrigger,
} from '@/modules/core/presentation/ui/shadcn';
import { Button } from '@/modules/core/presentation/ui/shadcn';
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@/modules/core/presentation/ui/shadcn';
import type { OrganizationUser } from '@/modules/user_settings/domain/models/OrganizationUser.ts';

interface EditRoleDialogProps {
  user: OrganizationUser;
  onUpdateRole: (newRole: string) => void;
  isLoading?: boolean;
}

export const EditRoleDialog: React.FC<EditRoleDialogProps> = ({
  user,
  onUpdateRole,
  isLoading,
}) => {
  const [open, setOpen] = useState(false);
  const [newRole, setNewRole] = useState(user.role);

  const handleSubmit = () => {
    if (newRole && newRole !== user.role) {
      onUpdateRole(newRole);
      setOpen(false);
    }
  };

  return (
    <Dialog open={open} onOpenChange={setOpen}>
      <DialogTrigger asChild>
        <Button variant='outline' size='sm' disabled={isLoading}>
          <Trans>Edit Role</Trans>
        </Button>
      </DialogTrigger>
      <DialogContent>
        <DialogHeader>
          <DialogTitle>
            <Trans>Update User Role</Trans>
          </DialogTitle>
          <DialogDescription>
            <Trans>Update the role for {user.username}</Trans>
          </DialogDescription>
        </DialogHeader>
        <div className='grid gap-4 py-4'>
          <div className='grid gap-2'>
            <label htmlFor='new-role'>
              <Trans>New Role</Trans>
            </label>
            <Select value={newRole} onValueChange={setNewRole} disabled={isLoading}>
              <SelectTrigger id='new-role'>
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
        </div>
        <div className='flex justify-end gap-2'>
          <Button variant='outline' onClick={() => setOpen(false)} disabled={isLoading}>
            <Trans>Cancel</Trans>
          </Button>
          <Button
            onClick={handleSubmit}
            disabled={!newRole || newRole === user.role || isLoading}
          >
            <Trans>Update Role</Trans>
          </Button>
        </div>
      </DialogContent>
    </Dialog>
  );
};
