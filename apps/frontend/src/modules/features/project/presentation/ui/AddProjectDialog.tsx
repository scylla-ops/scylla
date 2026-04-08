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
import { useEffect } from 'react';
import { useCreateProject } from '@/modules/features/project/presentation/hooks/useCreateProject.ts';
import { useContextStore } from '@shared/presentation/stores/useContext.ts';

interface AddOrganizationDialogProps {
  open: boolean;
  setOpen: (open: boolean) => void;
}

export function AddProjectDialog({ open, setOpen }: AddOrganizationDialogProps) {
  const [projectName, setProjectName] = React.useState('');
  const [isLoading, setIsLoading] = React.useState(false);
  const [error, setError] = React.useState<string | null>(null);
  const createProject = useCreateProject();
  const organizationId = useContextStore(state => state.organization.id);

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();

    const canCreate = projectName.trim() && organizationId;

    if (!canCreate) {
      setError('Project name is required and you must select to an organization.');
      return;
    }

    setIsLoading(true);
    setError(null);

    createProject.mutate(
      { name: projectName, organizationId: organizationId },
      {
        onSuccess: () => {
          setIsLoading(false);
          setOpen(false);
        },
        onError: err => {
          setIsLoading(false);
          setError(err.message || 'Failed to create project. Please try again.');
        },
      },
    );
  };

  useEffect(() => {
    setProjectName('');
    setError(null);
  }, [open]);

  return (
    <Dialog open={open} onOpenChange={setOpen}>
      <DialogContent>
        <DialogHeader>
          <DialogTitle>Create a new project</DialogTitle>
          <DialogDescription>
            Enter a name and description for your new project. You can change these later in
            settings.
          </DialogDescription>
        </DialogHeader>
        <form onSubmit={handleSubmit} className='space-y-4'>
          {error && (
            <div className='rounded-md bg-destructive/10 p-3 text-sm text-destructive'>{error}</div>
          )}
          <div className='space-y-2'>
            <Label htmlFor='project-name'>Project name</Label>
            <Input
              id='project-name'
              placeholder='e.g., My project'
              value={projectName}
              onChange={e => setProjectName(e.target.value)}
              autoFocus
              disabled={isLoading}
            />
          </div>
          <DialogFooter>
            <Button
              type='button'
              variant='outline'
              onClick={() => {
                setProjectName('');
                setError(null);
                setOpen(false);
              }}
              disabled={isLoading}
            >
              Cancel
            </Button>
            <Button type='submit' disabled={!projectName.trim() || isLoading}>
              {isLoading ? 'Creating...' : 'Create Project'}
            </Button>
          </DialogFooter>
        </form>
      </DialogContent>
    </Dialog>
  );
}
