import { useProjects } from '@/modules/features/project/presentation/hooks/useProjects.ts';
import ProjectCard from '@/modules/features/project/presentation/ui/ProjectCard.tsx';
import ProjectCardSkeleton from '@/modules/features/project/presentation/ui/ProjectCardSkeleton.tsx';
import { Skeleton } from '@/modules/shared/presentation/ui/shadcn/skeleton.tsx';
import { useDelayedLoading } from '@/modules/shared/presentation/hooks/useDelayedLoading.ts';

export const ProjectPage = () => {
  const { projects, isLoading, isError } = useProjects();
  const showSkeleton = useDelayedLoading(400);

  if (isLoading && !showSkeleton) {
    return <></>;
  }

  if (isLoading && showSkeleton) {
    return (
      <div className='flex flex-col gap-4 w-full h-full p-2'>
        <div className='flex items-baseline gap-2'>
          <Skeleton className='h-9 w-48' />
          <Skeleton className='h-5 w-16' />
        </div>
        <div className='grid grid-cols-1 sm:grid-cols-2 md:grid-cols-3 lg:grid-cols-4 gap-4'>
          {Array.from({ length: 8 }).map((_, i) => (
            <ProjectCardSkeleton key={i} />
          ))}
        </div>
      </div>
    );
  }

  if (isError || !projects) {
    return (
      <div className='flex items-center justify-center h-full'>
        <div className='text-center space-y-2'>
          <p className='text-destructive text-lg font-semibold'>Erreur</p>
          <p className='text-muted-foreground text-sm'>Impossible de charger les projets</p>
        </div>
      </div>
    );
  }

  return (
    <div className='flex flex-col gap-4 w-full h-full p-2'>
      <div className='flex items-baseline gap-2'>
        <h1 className='text-3xl font-bold tracking-tight'>
          <span className='text-primary'>{projects.length}</span>{' '}
          <span className='text-foreground'>
            Project
            {projects.length > 1 ? 's' : ''}
          </span>
        </h1>
        <span className='text-sm text-muted-foreground font-medium'>in total</span>
      </div>
      <div className='grid grid-cols-1 sm:grid-cols-2 md:grid-cols-3 lg:grid-cols-4 gap-4'>
        {projects &&
          projects.map(project => <ProjectCard key={project.projectId} project={project} />)}
      </div>
    </div>
  );
};

export default ProjectPage;
