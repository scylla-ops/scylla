import { Card, CardContent, CardHeader, CardTitle } from '@shadcn';
import { Folder, Pencil } from 'lucide-react';
import { useScyllaNavigate } from '@shared/presentation/hooks/use-scylla-navigate.ts';
import { Trans } from '@lingui/react/macro';
import type { ProjectEntity } from '@/modules/features/project/domain/entities/project.entity.ts';
import { useSelection } from '@shared/presentation/hooks/use-selection.ts';
import { EditProjectDialog } from '@/modules/features/project/presentation/ui/EditProjectDialog.tsx';
import { useState } from 'react';
import { cn } from '@shared/presentation/utils';
import { Checkbox } from '@shadcn/checkbox.tsx';
import { Tooltip, TooltipContent, TooltipTrigger } from '@shadcn/tooltip.tsx';
import { IconButton } from '@shared/presentation/ui';
import { Permission } from '@/modules/features/permission/domain/structs/permission.struct.ts';
import { Can } from '@/modules/features/permission/presentation/ui/authorization/Can.tsx';

type ProjectCardProps = {
  project: ProjectEntity;
};

export const ProjectCard = ({ project }: ProjectCardProps) => {
  const navigate = useScyllaNavigate();
  const { selectedIds, select } = useSelection('projects');
  const isSelected = selectedIds.includes(project.id);
  const [editOpen, setEditOpen] = useState(false);

  return (
    <>
      <Card
        onClick={() => navigate.goToProject(project)}
        className={cn(
          'group cursor-pointer transition-all duration-200 hover:shadow-lg hover:border-primary/50 active:scale-[0.98] h-full flex flex-col',
          isSelected && 'ring-2 ring-primary border-primary',
        )}
      >
        <CardHeader className='pb-2 overflow-hidden'>
          <div className='flex items-start justify-between gap-2 overflow-hidden'>
            <div className='flex items-center gap-3 min-w-0 flex-1 overflow-hidden'>
              <div className='rounded-lg bg-primary/10 p-2 group-hover:bg-primary/20 transition-colors shrink-0'>
                <Folder className='h-5 w-5 text-primary' />
              </div>
              <CardTitle className='text-base font-semibold truncate min-w-0' title={project.name}>
                {project.name}
              </CardTitle>
            </div>
            <div className='flex flex-row items-center gap-1 shrink-0'>
              {/* Checked against this card's project, not the active one. */}
              <Can permission={Permission.UPDATE_PROJECT} target={{ projectId: project.id }}>
                <IconButton
                  icon={Pencil}
                  tooltip={<Trans>Edit</Trans>}
                  onClick={e => {
                    e.stopPropagation();
                    setEditOpen(true);
                  }}
                  className='h-7 w-7 opacity-0 group-hover:opacity-100'
                  iconClassName='h-3.5 w-3.5'
                />
              </Can>
              <Tooltip>
                <TooltipTrigger asChild>
                  <span className={'mr-2'}>
                    <Checkbox
                      checked={isSelected}
                      className='transition-all cursor-pointer hover:scale-125 hover:border-primary'
                      onClick={e => e.stopPropagation()}
                      onCheckedChange={() => select(project.id)}
                    />
                  </span>
                </TooltipTrigger>
                <TooltipContent>
                  <p>
                    <Trans>Select</Trans>
                  </p>
                </TooltipContent>
              </Tooltip>
            </div>
          </div>
        </CardHeader>

        <CardContent className='flex-1 pt-0'>
          <p className='text-sm text-muted-foreground line-clamp-2'>
            {project.description || (
              <span className='italic'>
                <Trans>No description</Trans>
              </span>
            )}
          </p>
        </CardContent>
      </Card>
      <EditProjectDialog open={editOpen} setOpen={setEditOpen} project={project} />
    </>
  );
};

export default ProjectCard;
