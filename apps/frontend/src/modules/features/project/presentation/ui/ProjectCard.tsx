import { Card, CardContent, CardHeader, CardTitle } from '@shadcn';
import { Folder, Pencil } from 'lucide-react';
import { Button } from '@shadcn';
import { useScyllaNavigate } from '@shared/presentation/hooks/use-scylla-navigate.ts';
import { Trans } from '@lingui/react/macro';
import type { Project } from '@/modules/features/project/domain/models/project.model.ts';
import { useSelection } from '@shared/presentation/hooks/use-selection.ts';
import { EditProjectDialog } from '@/modules/features/project/presentation/ui/EditProjectDialog.tsx';
import { useState } from 'react';
import { cn } from '@shared/presentation/utils';

type ProjectCardProps = {
  project: Project;
};

export const ProjectCard = ({ project }: ProjectCardProps) => {
  const navigate = useScyllaNavigate();
  const { selectedIds, select } = useSelection('projects');
  const isSelected = selectedIds.includes(project.id);
  const [editOpen, setEditOpen] = useState(false);

  return (
    <>
      <Card
        onClick={() => navigate.goToProject({ id: project.id, name: project.name })}
        className={cn(
          'group cursor-pointer transition-all duration-200 hover:shadow-lg hover:border-primary/50 active:scale-[0.98] h-full',
          isSelected && 'ring-2 ring-primary border-primary',
        )}
      >
        <CardHeader className='space-y-0 pb-3 overflow-hidden'>
          <div className='flex items-start justify-between gap-3 overflow-hidden'>
            <div className='flex items-center gap-3 min-w-0 flex-1 overflow-hidden'>
              <div className='rounded-lg bg-primary/10 p-2.5 group-hover:bg-primary/20 transition-colors'>
                <Folder className='h-5 w-5 text-primary' />
              </div>
              <CardTitle className='text-lg font-semibold truncate min-w-0' title={project.name}>
                {project.name}
              </CardTitle>
            </div>
            <div className='flex gap-1'>
              <Button
                size='icon'
                variant='ghost'
                className='h-8 w-8 opacity-0 group-hover:opacity-100 transition-opacity'
                onClick={e => {
                  e.stopPropagation();
                  setEditOpen(true);
                }}
              >
                <Pencil className='h-4 w-4' />
              </Button>
              <input
                type='checkbox'
                checked={isSelected}
                className='accent-primary h-4 w-4 mt-1 cursor-pointer'
                onClick={e => e.stopPropagation()}
                onChange={() => select(project.id)}
              />
            </div>
          </div>
        </CardHeader>

        <CardContent className='space-y-3'>
          <div className='pt-2 border-t border-border/50'>
            <div className='flex items-center justify-between text-xs text-muted-foreground'>
              <span className='flex items-center gap-1'>
                <span className='font-medium text-foreground'>
                  <Trans>Project</Trans>
                </span>
              </span>
            </div>
          </div>
        </CardContent>
      </Card>
      <EditProjectDialog open={editOpen} setOpen={setEditOpen} project={project} />
    </>
  );
};

export default ProjectCard;
