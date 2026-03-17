import type { ComponentType, ReactNode } from 'react';
import { useProjects } from '@/modules/features/project/presentation/hooks/useProjects.ts';
import { ContextItem } from '@/modules/layout/presentation/ui/context-selector/ContextItem.tsx';
import { useContextStore } from '@/modules/shared/presentation/stores/useContext.ts';
import { FolderKanban } from 'lucide-react';

interface ProjectListProps {
  Wrapper: ComponentType<{ children: ReactNode; onSelect?: () => void }>;
}

export const ProjectList = ({ Wrapper }: ProjectListProps) => {
  const { projects } = useProjects();
  const setProject = useContextStore(state => state.setProject);
  const organizationId = useContextStore(state => state.organization.id);

  if (!organizationId) return <p>No organization selected</p>;

  if (!projects) return <p>Loading...</p>;

  if (projects.length === 0) return <p>No projects found. Please create a project.</p>;

  return (
    <>
      {projects.map(project => (
        <Wrapper
          key={project.projectId}
          onSelect={() => setProject(project.projectId, project.name)}
        >
          <ContextItem name={project.name} icon={FolderKanban} />
        </Wrapper>
      ))}
    </>
  );
};

export default ProjectList;
