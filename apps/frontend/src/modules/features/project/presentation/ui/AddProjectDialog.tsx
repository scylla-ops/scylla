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
import { useCreateProject } from '@/modules/features/project/presentation/hooks/useCreateProject.ts';
import { useContextStore } from '@/modules/shared/presentation/stores/useContext.ts';

interface AddProjectDialogProps {
  open: boolean;
  setOpen: (open: boolean) => void;
}

export function AddProjectDialog({ open, setOpen }: AddProjectDialogProps) {
  const [projectName, setProjectName] = React.useState('');
  const [description, setDescription] = React.useState('');
  const [isLoading, setIsLoading] = React.useState(false);
  const [error, setError] = React.useState<string | null>(null);
  const createProject = useCreateProject();

  const organizationId = useContextStore(state => state.organization.id);

  useEffect(() => {
    setProjectName('');
    setDescription('');
    setError(null);
  }, [open]);

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!projectName.trim()) return;

    setIsLoading(true);
    setError(null);

    if (!organizationId) {
      setError('Organization ID is required');
      setIsLoading(false);
      return;
    }

    createProject.mutate(
      { name: projectName, organizationId: organizationId },
      {
        onSuccess: () => {
          setIsLoading(false);
          setOpen(false);
        },
        onError: err => {
          setIsLoading(false);
          setError(err.message || 'Failed to create organization. Please try again.');
        },
      },
    );
  };

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
              placeholder='e.g., My Project'
              value={projectName}
              onChange={e => setProjectName(e.target.value)}
              autoFocus
              disabled={isLoading}
            />
          </div>
          <div className='space-y-2'>
            <Label htmlFor='project-description'>Description</Label>
            <Input
              id='project-description'
              placeholder="e.g., Our company's main project"
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
                setProjectName('');
                setDescription('');
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

export default AddProjectDialog;
