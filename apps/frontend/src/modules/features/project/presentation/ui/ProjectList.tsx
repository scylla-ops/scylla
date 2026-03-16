import type { ComponentType, ReactNode } from 'react';
import { useProjects } from '@/modules/features/project/presentation/hooks/useProjects.ts';
import { AvailableProjectItem } from '@/modules/features/project/presentation/ui/AvaiableProjectItem.tsx';
import { useProjectStore } from '@/modules/features/project/presentation/stores/useProjectStore.ts';
import { useContextStore } from '@/modules/shared/presentation/stores/useContext.ts';

interface ProjectListProps {
  Wrapper: ComponentType<{ children: ReactNode; onSelect?: () => void }>;
}

export const ProjectList = ({ Wrapper }: ProjectListProps) => {
  const { projects } = useProjects();
  const setProjectName = useProjectStore(state => state.setCurrentProjectName);
  const setProjectId = useContextStore(state => state.setProjectId);

  const organizationId = useContextStore(state => state.organizationId);

  if (!organizationId) return <p>No organization selected</p>;

  if (!projects) return <p>Loading...</p>;

  if (projects.length === 0) return <p>No projects found. Please create a project.</p>;

  return (
    <>
      {projects.map(project => (
        <Wrapper
          key={project.projectId}
          onSelect={() => {
            setProjectId(project.projectId);
            setProjectName(project.name);
          }}
        >
          <AvailableProjectItem name={project.name} />
        </Wrapper>
      ))}
    </>
  );
};

export default ProjectList;
