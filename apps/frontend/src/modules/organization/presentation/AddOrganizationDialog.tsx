'use client';

import * as React from 'react';
import { Button } from '@shadcn/button.tsx';
import { Input } from '@shadcn/input.tsx';
import { Label } from '@shadcn/label.tsx';
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from '@shadcn/dialog.tsx';

interface AddOrganizationDialogProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  onAddOrganization: (organization: { name: string; description: string }) => Promise<void>;
}

export function AddOrganizationDialog({
  open,
  onOpenChange,
  onAddOrganization,
}: AddOrganizationDialogProps) {
  const [organizationName, setOrganizationName] = React.useState('');
  const [description, setDescription] = React.useState('');
  const [isLoading, setIsLoading] = React.useState(false);
  const [error, setError] = React.useState<string | null>(null);

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!organizationName.trim()) return;

    setIsLoading(true);
    setError(null);

    try {
      // const tokenStore = getCoreMemoryStore();
      // const transport = new CoreGrpcTransport(tokenStore);
      // const client = new OrganizationServiceClient(transport.getTransport());

      // const request: CreateOrganizationRequest = {
      //   name: organizationName,
      //   description: description.trim() || undefined,
      // };

      // await client.createOrganization(request);

      await onAddOrganization({
        name: organizationName,
        description: description.trim(),
      });

      setOrganizationName('');
      setDescription('');
      onOpenChange(false);
    } catch (err) {
      const errorMessage = err instanceof Error ? err.message : 'Failed to create organization';
      setError(errorMessage);
      console.error('Error creating organization:', err);
    } finally {
      setIsLoading(false);
    }
  };

  const handleOpenChange = (newOpen: boolean) => {
    if (!isLoading) {
      onOpenChange(newOpen);
      if (!newOpen) {
        setOrganizationName('');
        setDescription('');
        setError(null);
      }
    }
  };

  return (
    <Dialog open={open} onOpenChange={handleOpenChange}>
      <DialogContent>
        <DialogHeader>
          <DialogTitle>Create a new organization</DialogTitle>
          <DialogDescription>
            Enter a name and description for your new organization. You can change these later in
            settings.
          </DialogDescription>
        </DialogHeader>
        <form onSubmit={handleSubmit} className='space-y-4'>
          {error && (
            <div className='rounded-md bg-destructive/10 p-3 text-sm text-destructive'>{error}</div>
          )}
          <div className='space-y-2'>
            <Label htmlFor='organization-name'>Organization name</Label>
            <Input
              id='organization-name'
              placeholder='e.g., My Organization'
              value={organizationName}
              onChange={e => setOrganizationName(e.target.value)}
              autoFocus
              disabled={isLoading}
            />
          </div>
          <div className='space-y-2'>
            <Label htmlFor='organization-description'>Description</Label>
            <Input
              id='organization-description'
              placeholder="e.g., Our company's main organization"
              value={description}
              onChange={e => setDescription(e.target.value)}
              disabled={isLoading}
            />
          </div>
          <DialogFooter>
            <Button
              type='button'
              variant='outline'
              onClick={() => {
                setOrganizationName('');
                setDescription('');
                setError(null);
                onOpenChange(false);
              }}
              disabled={isLoading}
            >
              Cancel
            </Button>
            <Button type='submit' disabled={!organizationName.trim() || isLoading}>
              {isLoading ? 'Creating...' : 'Create Organization'}
            </Button>
          </DialogFooter>
        </form>
      </DialogContent>
    </Dialog>
  );
}
