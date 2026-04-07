import type { ProjectResponse } from '@/generated/project.ts';
import { Card, CardContent, CardHeader, CardTitle } from '@shadcn';
import { useNavigate } from 'react-router-dom';
import { Folder } from 'lucide-react';

type ProjectCardProps = {
  project: ProjectResponse;
};

export const ProjectCard = ({ project }: ProjectCardProps) => {
  const navigate = useNavigate();

  return (
    <Card
      onClick={() => navigate(`/projects/${project.projectId}`)}
      className='group cursor-pointer transition-all duration-200 hover:shadow-lg hover:border-primary/50 active:scale-[0.98] h-full'
    >
      <CardHeader className='space-y-0 pb-3'>
        <div className='flex items-start justify-between gap-3'>
          <div className='flex items-center gap-3 min-w-0 flex-1'>
            <div className='rounded-lg bg-primary/10 p-2.5 group-hover:bg-primary/20 transition-colors'>
              <Folder className='h-5 w-5 text-primary' />
            </div>
            <CardTitle className='text-lg font-semibold truncate'>{project.name}</CardTitle>
          </div>
        </div>
      </CardHeader>

      <CardContent className='space-y-3'>
        <p className='text-sm text-muted-foreground line-clamp-2 min-h-[2.5rem]'>
          {project.description || 'Aucune description'}
        </p>

        <div className='pt-2 border-t border-border/50'>
          <div className='flex items-center justify-between text-xs text-muted-foreground'>
            <span className='flex items-center gap-1'>
              <span className='font-medium text-foreground'>Projet</span>
            </span>
          </div>
        </div>
      </CardContent>
    </Card>
  );
};

export default ProjectCard;
