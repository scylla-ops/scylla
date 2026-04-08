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
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@shadcn/select.tsx';
import { useEffect } from 'react';
import { useCreateProject } from '@/modules/features/project/presentation/hooks/useCreateProject.ts';
import { useOrganizations } from '@/modules/features/organization/presentation/hooks/useOrganizations.ts';
import { useContextStore } from '@shared/presentation/stores/useContext.ts';
import { toast } from '@shared/presentation/utils/toast.ts';

interface AddProjectDialogProps {
  open: boolean;
  setOpen: (open: boolean) => void;
}

export function AddProjectDialog({ open, setOpen }: AddProjectDialogProps) {
  const [projectName, setProjectName] = React.useState('');
  const createProject = useCreateProject();
  const contextOrg = useContextStore(state => state.organization);
  const { organizations } = useOrganizations();
  const [selectedOrgId, setSelectedOrgId] = React.useState<string | null>(contextOrg.id);

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();

    if (!projectName.trim() || !selectedOrgId) {
      toast.error('Project name is required and you must select an organization.');
      return;
    }

    createProject.mutate(
      { name: projectName, organizationId: selectedOrgId },
      {
        onSuccess: () => setOpen(false),
      },
    );
  };

  useEffect(() => {
    setProjectName('');
    setSelectedOrgId(contextOrg.id);
  }, [open, contextOrg.id]);

  return (
    <Dialog open={open} onOpenChange={setOpen}>
      <DialogContent>
        <DialogHeader>
          <DialogTitle>Create a new project</DialogTitle>
          <DialogDescription>
            Enter a name for your new project and select the organization it belongs to.
          </DialogDescription>
        </DialogHeader>
        <form onSubmit={handleSubmit} className='space-y-4'>
          <div className='space-y-2'>
            <Label htmlFor='project-org'>Organization</Label>
            <Select
              value={selectedOrgId ?? undefined}
              onValueChange={setSelectedOrgId}
              disabled={createProject.isPending}
            >
              <SelectTrigger id='project-org' className='w-full'>
                <SelectValue placeholder='Select an organization' />
              </SelectTrigger>
              <SelectContent>
                {organizations?.map(org => (
                  <SelectItem key={org.organizationId} value={org.organizationId}>
                    {org.name}
                  </SelectItem>
                ))}
              </SelectContent>
            </Select>
          </div>
          <div className='space-y-2'>
            <Label htmlFor='project-name'>Project name</Label>
            <Input
              id='project-name'
              placeholder='e.g., My project'
              value={projectName}
              onChange={e => setProjectName(e.target.value)}
              autoFocus
              disabled={createProject.isPending}
            />
          </div>
          <DialogFooter>
            <Button
              type='button'
              variant='outline'
              onClick={() => {
                setProjectName('');
                setOpen(false);
              }}
              disabled={createProject.isPending}
            >
              Cancel
            </Button>
            <Button
              type='submit'
              disabled={!projectName.trim() || !selectedOrgId || createProject.isPending}
            >
              {createProject.isPending ? 'Creating...' : 'Create Project'}
            </Button>
          </DialogFooter>
        </form>
      </DialogContent>
    </Dialog>
  );
}
