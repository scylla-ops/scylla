import { useProjects } from '@/modules/features/project/presentation/hooks/useProjects.ts';
import ProjectCard from '@/modules/features/project/presentation/ui/ProjectCard.tsx';
import ProjectCardSkeleton from '@/modules/features/project/presentation/ui/ProjectCardSkeleton.tsx';
import { Skeleton } from '@/modules/shared/presentation/ui/shadcn/skeleton.tsx';
import { useDelayedLoading } from '@/modules/shared/presentation/hooks/useDelayedLoading.ts';
import { ProjectHeader } from '@/modules/features/project/presentation/ui/ProjectHeader.tsx';
import { useContextStore } from '@/modules/shared/presentation/stores/useContext.ts';
import { Pagination } from '@/modules/shared/presentation/ui/Pagination.tsx';

export const ProjectPage = () => {
  const organizationId = useContextStore(state => state.organization.id);
  const { projects, isLoading, isError, paginationInfo, setPage } = useProjects(organizationId);
  const showSkeleton = useDelayedLoading(400);

  if (!organizationId) {
    return (
      <div className='flex items-center justify-center h-full'>
        <div className='text-center space-y-2'>
          <p className='text-muted-foreground text-lg font-semibold'>No organization selected</p>
          <p className='text-sm text-muted-foreground'>
            Select an organization from the sidebar to view its projects
          </p>
        </div>
      </div>
    );
  }

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
    <div className='flex flex-col gap-4 w-full min-h-full p-2'>
      <ProjectHeader numberOfProjects={paginationInfo?.totalCount ?? projects.length} />
      <div className='grid grid-cols-1 sm:grid-cols-2 md:grid-cols-3 lg:grid-cols-4 gap-4'>
        {projects.map(project => (
          <ProjectCard key={project.projectId} project={project} />
        ))}
      </div>
      {paginationInfo && paginationInfo.totalPages > 1 && (
        <Pagination paginationInfo={paginationInfo} onPageChange={setPage} className='pb-2' />
      )}
    </div>
  );
};

export default ProjectPage;
