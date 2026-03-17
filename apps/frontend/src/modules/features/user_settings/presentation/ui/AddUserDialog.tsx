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
import { Input } from '@/modules/core/presentation/ui/shadcn';

interface AddUserDialogProps {
  onAddUser: (userId: string, role: string) => void;
  isLoading?: boolean;
}

export const AddUserDialog: React.FC<AddUserDialogProps> = ({ onAddUser, isLoading }) => {
  const [open, setOpen] = useState(false);
  const [userId, setUserId] = useState('');
  const [role, setRole] = useState('member');

  const handleSubmit = () => {
    if (userId && role) {
      onAddUser(userId, role);
      setUserId('');
      setRole('member');
      setOpen(false);
    }
  };

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
            <label htmlFor='user-id'>
              <Trans>User ID</Trans>
            </label>
            <Input
              id='user-id'
              placeholder='Enter user ID'
              value={userId}
              onChange={(e) => setUserId(e.target.value)}
              disabled={isLoading}
            />
          </div>
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
        </div>
        <div className='flex justify-end gap-2'>
          <Button variant='outline' onClick={() => setOpen(false)} disabled={isLoading}>
            <Trans>Cancel</Trans>
          </Button>
          <Button onClick={handleSubmit} disabled={!userId || !role || isLoading}>
            <Trans>Add User</Trans>
          </Button>
        </div>
      </DialogContent>
    </Dialog>
  );
};
