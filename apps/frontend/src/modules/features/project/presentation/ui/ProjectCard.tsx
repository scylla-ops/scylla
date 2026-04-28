import { Card, CardContent, CardHeader, CardTitle } from '@shadcn';
import { Folder } from 'lucide-react';
import { useScyllaNavigate } from '@shared/presentation/hooks/use-scylla-navigate.ts';
import { Trans } from '@lingui/react/macro';
import type { Project } from '@/modules/features/project/domain/models/project.model.ts';

type ProjectCardProps = {
  project: Project;
};

export const ProjectCard = ({ project }: ProjectCardProps) => {
  const navigate = useScyllaNavigate();

  return (
    <Card
      onClick={() => navigate.goToProject({ id: project.id, name: project.name })}
      className='group cursor-pointer transition-all duration-200 hover:shadow-lg hover:border-primary/50 active:scale-[0.98] h-full'
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
        </div>
      </CardHeader>

      <CardContent className='space-y-3'>
        <p className='text-sm text-muted-foreground line-clamp-2 min-h-[2.5rem]'>
          {<Trans>No description (coming soon)</Trans>}
        </p>

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
  );
};

export default ProjectCard;
