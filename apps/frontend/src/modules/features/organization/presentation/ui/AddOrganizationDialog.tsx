import * as React from 'react';
import { Button } from '@/modules/shared/presentation/ui/shadcn/button.tsx';
import { Input } from '@/modules/shared/presentation/ui/shadcn/input.tsx';
import { Label } from '@/modules/shared/presentation/ui/shadcn/label.tsx';
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from '@/modules/shared/presentation/ui/shadcn/dialog.tsx';
import { useEffect } from 'react';
import { useCreateOrganization } from '@/modules/features/organization/presentation/hooks/useCreateOrganization.ts';

interface AddOrganizationDialogProps {
  open: boolean;
  setOpen: (open: boolean) => void;
}

export function AddOrganizationDialog({ open, setOpen }: AddOrganizationDialogProps) {
  const [organizationName, setOrganizationName] = React.useState('');
  const [description, setDescription] = React.useState('');
  const createOrganization = useCreateOrganization();

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!organizationName.trim()) return;

    createOrganization.mutate(organizationName, {
      onSuccess: () => setOpen(false),
    });
  };

  useEffect(() => {
    setOrganizationName('');
    setDescription('');
  }, [open]);

  return (
    <Dialog open={open} onOpenChange={setOpen}>
      <DialogContent>
        <DialogHeader>
          <DialogTitle>Create a new organization</DialogTitle>
          <DialogDescription>
            Enter a name and description for your new organization. You can change these later in
            settings.
          </DialogDescription>
        </DialogHeader>
        <form onSubmit={handleSubmit} className='space-y-4'>
          <div className='space-y-2'>
            <Label htmlFor='organization-name'>Organization name</Label>
            <Input
              id='organization-name'
              placeholder='e.g., My Organization'
              value={organizationName}
              onChange={e => setOrganizationName(e.target.value)}
              autoFocus
              disabled={createOrganization.isPending}
            />
          </div>
          <div className='space-y-2'>
            <Label htmlFor='organization-description'>Description</Label>
            <Input
              id='organization-description'
              placeholder="e.g., Our company's main organization"
              value={description}
              onChange={e => setDescription(e.target.value)}
              disabled={createOrganization.isPending}
            />
          </div>
          <DialogFooter>
            <Button
              type='button'
              variant='outline'
              onClick={() => {
                setOrganizationName('');
                setDescription('');
                setOpen(false);
              }}
              disabled={createOrganization.isPending}
            >
              Cancel
            </Button>
            <Button
              type='submit'
              disabled={!organizationName.trim() || createOrganization.isPending}
            >
              {createOrganization.isPending ? 'Creating...' : 'Create Organization'}
            </Button>
          </DialogFooter>
        </form>
      </DialogContent>
    </Dialog>
  );
}
