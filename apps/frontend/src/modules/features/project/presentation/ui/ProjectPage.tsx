import { useProjects } from '@/modules/features/project/presentation/hooks/useProjects.ts';
import ProjectCard from '@/modules/features/project/presentation/ui/ProjectCard.tsx';
import { ProjectHeader } from '@/modules/features/project/presentation/ui/ProjectHeader.tsx';
import { useContextStore } from '@shared/presentation/stores/use-context.store.ts';
import { Pagination } from '@/modules/shared/presentation/ui/data-display/Pagination.tsx';
import { ErrorState } from '@/modules/shared/presentation/ui/feedback/ErrorState.tsx';
import { Trans } from '@lingui/react/macro';

export const ProjectPage = () => {
  const organizationId = useContextStore(state => state.organization.id);
  const { projects, isLoading, isError, paginationInfo, setPage } = useProjects(organizationId);

  if (!organizationId) {
    return (
      <div className='flex items-center justify-center h-full'>
        <div className='text-center space-y-2'>
          <p className='text-muted-foreground text-lg font-semibold'>
            <Trans>No organization selected</Trans>
          </p>
          <p className='text-sm text-muted-foreground'>
            <Trans>Select an organization from the sidebar to view its projects</Trans>
          </p>
        </div>
      </div>
    );
  }

  if (isLoading) {
    return <></>;
  }

  if (isError || !projects) {
    return <ErrorState message='Unable to load projects' />;
  }

  return (
    <div className='flex flex-col gap-4 w-full min-h-full'>
      <ProjectHeader
        numberOfProjects={paginationInfo?.totalCount ?? projects.length}
        projectIds={projects.map(project => project.id)}
      />
      <div className='grid grid-cols-1 sm:grid-cols-2 md:grid-cols-3 lg:grid-cols-4 gap-4'>
        {projects.map(project => (
          <ProjectCard key={project.id} project={project} />
        ))}
      </div>
      {paginationInfo && paginationInfo.totalPages > 1 && (
        <Pagination paginationInfo={paginationInfo} onPageChange={setPage} className='pb-2' />
      )}
    </div>
  );
};

export default ProjectPage;
