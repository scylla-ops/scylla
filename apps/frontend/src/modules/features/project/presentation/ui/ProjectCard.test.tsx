import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen } from '@testing-library/react';
import { createProvidersWrapper } from '@/test/render.tsx';
import userEvent from '@testing-library/user-event';
import { useContextStore } from '@platform/context';
import { usePermissionsStore, PermissionScope } from '@platform/authz';
import { useSelectionStore } from '@shared/presentation/stores/use-selection.store.ts';
import { ProjectCard } from './ProjectCard';
import type { ProjectEntity } from '@/modules/features/project/domain/entities/project.entity.ts';
import type { ProjectRepository } from '@/modules/features/project/domain/repository/project.repository.ts';
import { ScyllaResult } from '@shared/utils/scylla-result.ts';

const navigateMock = vi.fn();
vi.mock('react-router-dom', () => ({
  useNavigate: () => navigateMock,
  useLocation: () => ({ pathname: '/acme/projects' }),
}));

const project = (overrides: Partial<ProjectEntity> = {}): ProjectEntity => ({
  id: 'project-1',
  name: 'web',
  description: 'the web app',
  ...overrides,
});

const fakeProjectRepository = (): ProjectRepository => ({
  getByOrganizationId: vi.fn(),
  listMembers: vi.fn(),
  create: vi.fn(),
  update: vi.fn().mockResolvedValue(ScyllaResult.success(project())),
  delete: vi.fn(),
});

const wrapperFor = (repository: ProjectRepository) => createProvidersWrapper({ project: { projectRepository: repository } }).Wrapper;

const renderCard = (ui: React.ReactElement) => {
  const Wrapper = wrapperFor(fakeProjectRepository());
  return render(ui, { wrapper: Wrapper });
};

beforeEach(() => {
  navigateMock.mockClear();
  useSelectionStore.setState({ selectedIds: {} });
  useContextStore.setState({
    organization: { id: 'org-1', name: 'Acme' },
    project: { id: null, name: null },
  });
  usePermissionsStore.setState({
    permissions: { scopes: [{ scope: PermissionScope.SYSTEM, scopeId: '', access: { kind: 'fullControl' } }] },
  });
});

describe('ProjectCard', () => {
  it('navigates to the project when clicked', async () => {
    const user = userEvent.setup();
    renderCard(<ProjectCard project={project()} />);
    await user.click(screen.getByText('web'));
    expect(navigateMock).toHaveBeenCalled();
  });

  it('shows the description', () => {
    renderCard(<ProjectCard project={project({ description: 'a real one' })} />);
    expect(screen.getByText('a real one')).toBeInTheDocument();
  });

  it('shows an italic "No description" placeholder when there is none', () => {
    renderCard(<ProjectCard project={project({ description: '' })} />);
    expect(screen.getByText('No description')).toBeInTheDocument();
  });

  // IconButton renders its tooltip text visually-hidden inside the button, so
  // the Edit affordance carries a real accessible name.
  const findEditButton = () => screen.queryByRole('button', { name: 'Edit' });

  it('the Edit button is hidden without UPDATE_PROJECT on this project', () => {
    usePermissionsStore.setState({ permissions: { scopes: [] } });
    renderCard(<ProjectCard project={project()} />);
    expect(findEditButton()).toBeNull();
  });

  it('the Edit button opens the edit dialog without navigating', async () => {
    const user = userEvent.setup();
    renderCard(<ProjectCard project={project()} />);

    await user.click(findEditButton()!);

    expect(navigateMock).not.toHaveBeenCalled();
    expect(screen.getByText('Edit project')).toBeInTheDocument();
  });

  it('checking the selection checkbox toggles selection without navigating', async () => {
    const user = userEvent.setup();
    renderCard(<ProjectCard project={project({ id: 'project-9' })} />);

    await user.click(screen.getByRole('checkbox'));

    expect(navigateMock).not.toHaveBeenCalled();
    expect(useSelectionStore.getState().selectedIds.projects).toContain('project-9');
  });
});
