import { describe, it, expect, vi, beforeEach } from 'vitest';
import { screen } from '@testing-library/react';
import { renderWithProviders } from '@/test/render.tsx';
import { stubQuery } from '@/test/queries.ts';
import { useContextStore } from '@platform/context';
import { ContextCleanerWrapper } from './ContextCleaner.wrapper';

const paramsMock = vi.fn<() => { projectId?: string }>(() => ({}));
const navigateMock = vi.fn();
const locationMock = vi.fn(() => ({ pathname: '/acme/projects/project-1' }));
vi.mock('react-router-dom', () => ({
  useParams: () => paramsMock(),
  useNavigate: () => navigateMock,
  useLocation: () => locationMock(),
  Outlet: () => <div data-testid='outlet' />,
}));

const projectsState: { projects?: { id: string; name: string }[]; isLoading: boolean } = {
  projects: undefined,
  isLoading: false,
};
vi.mock('@/modules/features/project', () => ({
  projectQueries: {
    byOrganization: () =>
      stubQuery(
        ['projects'],
        projectsState.projects && { projects: projectsState.projects },
        { loading: projectsState.isLoading },
      ),
  },
}));

beforeEach(() => {
  paramsMock.mockReturnValue({});
  navigateMock.mockClear();
  locationMock.mockReturnValue({ pathname: '/acme/projects/project-1' });
  projectsState.projects = [{ id: 'project-1', name: 'web' }];
  projectsState.isLoading = false;
  useContextStore.setState({
    organization: { id: 'org-1', name: 'Acme Corp' },
    project: { id: 'project-1', name: 'web' },
    pipeline: { id: null, name: null },
  });
});

describe('ContextCleanerWrapper', () => {
  it('always renders the outlet', () => {
    renderWithProviders(<ContextCleanerWrapper />);
    expect(screen.getByTestId('outlet')).toBeInTheDocument();
  });

  it('does nothing while projects are still loading', () => {
    paramsMock.mockReturnValue({ projectId: 'project-1' });
    projectsState.isLoading = true;
    renderWithProviders(<ContextCleanerWrapper />);
    expect(navigateMock).not.toHaveBeenCalled();
  });

  it('does nothing without a projectId param', () => {
    renderWithProviders(<ContextCleanerWrapper />);
    expect(navigateMock).not.toHaveBeenCalled();
    expect(useContextStore.getState().project.id).toBe('project-1');
  });

  it('leaves an existing project alone when it is still in the list', () => {
    paramsMock.mockReturnValue({ projectId: 'project-1' });
    renderWithProviders(<ContextCleanerWrapper />);
    expect(navigateMock).not.toHaveBeenCalled();
    expect(useContextStore.getState().project).toEqual({ id: 'project-1', name: 'web' });
  });

  it('clears project and pipeline and navigates back to the org\'s projects list when the project no longer exists', () => {
    paramsMock.mockReturnValue({ projectId: 'deleted-project' });
    renderWithProviders(<ContextCleanerWrapper />);
    expect(useContextStore.getState().project).toEqual({ id: null, name: null });
    expect(useContextStore.getState().pipeline).toEqual({ id: null, name: null });
    expect(navigateMock).toHaveBeenCalledWith('/acme-corp/projects', { replace: true });
  });

  it('falls back to "/" when there is no organization name to slugify', () => {
    useContextStore.setState({ organization: { id: null, name: null } });
    paramsMock.mockReturnValue({ projectId: 'deleted-project' });
    renderWithProviders(<ContextCleanerWrapper />);
    expect(navigateMock).toHaveBeenCalledWith('/', { replace: true });
  });

  it('drops a stale pipeline when navigating away from the pipeline editor/detail routes', () => {
    useContextStore.setState({ pipeline: { id: 'pipeline-1', name: 'ci' } });
    locationMock.mockReturnValue({ pathname: '/acme/projects/project-1/settings' });
    paramsMock.mockReturnValue({ projectId: 'project-1' });
    renderWithProviders(<ContextCleanerWrapper />);
    expect(useContextStore.getState().pipeline).toEqual({ id: null, name: null });
  });

  it('keeps the pipeline while still on its editor route', () => {
    useContextStore.setState({ pipeline: { id: 'pipeline-1', name: 'ci' } });
    locationMock.mockReturnValue({ pathname: '/acme/projects/project-1/pipelines/pipeline-1/edit/' });
    paramsMock.mockReturnValue({ projectId: 'project-1' });
    renderWithProviders(<ContextCleanerWrapper />);
    expect(useContextStore.getState().pipeline).toEqual({ id: 'pipeline-1', name: 'ci' });
  });

  it('keeps the pipeline while on any /pipelines/ route even outside the editor', () => {
    useContextStore.setState({ pipeline: { id: 'pipeline-1', name: 'ci' } });
    locationMock.mockReturnValue({ pathname: '/acme/projects/project-1/pipelines/pipeline-1' });
    paramsMock.mockReturnValue({ projectId: 'project-1' });
    renderWithProviders(<ContextCleanerWrapper />);
    expect(useContextStore.getState().pipeline).toEqual({ id: 'pipeline-1', name: 'ci' });
  });
});
