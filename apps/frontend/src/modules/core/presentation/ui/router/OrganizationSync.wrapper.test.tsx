import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen } from '@testing-library/react';
import { useContextStore } from '@platform/context';
import { OrganizationSyncWrapper } from './OrganizationSync.wrapper';

const paramsMock = vi.fn<() => { organizationSlug?: string }>(() => ({}));
const navigateMock = vi.fn();
vi.mock('react-router-dom', () => ({
  useParams: () => paramsMock(),
  useNavigate: () => navigateMock,
  Outlet: () => <div data-testid='outlet' />,
}));

const organizationsState: { organizations?: { id: string; name: string }[]; isLoading: boolean } = {
  organizations: undefined,
  isLoading: false,
};
vi.mock('@/modules/features/organization', () => ({
  useOrganizations: () => organizationsState,
}));

beforeEach(() => {
  paramsMock.mockReturnValue({});
  navigateMock.mockClear();
  organizationsState.organizations = [
    { id: 'org-1', name: 'Acme Corp' },
    { id: 'org-2', name: 'Globex Inc' },
  ];
  organizationsState.isLoading = false;
  useContextStore.setState({ organization: { id: null, name: null } });
});

describe('OrganizationSyncWrapper', () => {
  it('always renders the outlet', () => {
    render(<OrganizationSyncWrapper />);
    expect(screen.getByTestId('outlet')).toBeInTheDocument();
  });

  it('does nothing without a slug param', () => {
    render(<OrganizationSyncWrapper />);
    expect(useContextStore.getState().organization.id).toBeNull();
    expect(navigateMock).not.toHaveBeenCalled();
  });

  it('does nothing while organizations are still loading', () => {
    paramsMock.mockReturnValue({ organizationSlug: 'acme-corp' });
    organizationsState.isLoading = true;
    render(<OrganizationSyncWrapper />);
    expect(useContextStore.getState().organization.id).toBeNull();
    expect(navigateMock).not.toHaveBeenCalled();
  });

  it('adopts the organization matching the slug into the context store', () => {
    paramsMock.mockReturnValue({ organizationSlug: 'globex-inc' });
    render(<OrganizationSyncWrapper />);
    expect(useContextStore.getState().organization).toEqual({ id: 'org-2', name: 'Globex Inc' });
    expect(navigateMock).not.toHaveBeenCalled();
  });

  it('does not re-set the store when the matched org is already current', () => {
    useContextStore.setState({ organization: { id: 'org-1', name: 'Acme Corp' } });
    const setStateSpy = vi.spyOn(useContextStore, 'setState');
    paramsMock.mockReturnValue({ organizationSlug: 'acme-corp' });
    render(<OrganizationSyncWrapper />);
    expect(setStateSpy).not.toHaveBeenCalled();
  });

  it('falls back to the first organization and redirects when the slug matches none', () => {
    paramsMock.mockReturnValue({ organizationSlug: 'no-such-org' });
    render(<OrganizationSyncWrapper />);
    expect(navigateMock).toHaveBeenCalledWith('/acme-corp/dashboard', { replace: true });
    expect(useContextStore.getState().organization).toEqual({ id: 'org-1', name: 'Acme Corp' });
  });

  it('does nothing when the slug matches none and there are no organizations to fall back on', () => {
    paramsMock.mockReturnValue({ organizationSlug: 'no-such-org' });
    organizationsState.organizations = [];
    render(<OrganizationSyncWrapper />);
    expect(navigateMock).not.toHaveBeenCalled();
    expect(useContextStore.getState().organization.id).toBeNull();
  });
});
