import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen } from '@testing-library/react';
import { useContextStore } from '@platform/context';
import { OrganizationRedirectWrapper } from './OrganizationRedirect.wrapper';

vi.mock('react-router-dom', () => ({
  Navigate: ({ to }: { to: string }) => <div data-testid='navigate' data-to={to} />,
}));

const organizationsState: { organizations?: { id: string; name: string }[]; isLoading: boolean } = {
  organizations: undefined,
  isLoading: false,
};
vi.mock('@/modules/features/organization', () => ({
  useOrganizations: () => organizationsState,
}));

beforeEach(() => {
  organizationsState.organizations = undefined;
  organizationsState.isLoading = false;
  useContextStore.setState({ organization: { id: null, name: null } });
});

describe('OrganizationRedirectWrapper', () => {
  it('renders nothing while organizations are loading', () => {
    organizationsState.isLoading = true;
    const { container } = render(<OrganizationRedirectWrapper />);
    expect(container).toBeEmptyDOMElement();
  });

  it('redirects to the stored organization when one is already set', () => {
    useContextStore.setState({ organization: { id: 'org-1', name: 'Acme Corp' } });
    organizationsState.organizations = [{ id: 'org-2', name: 'Other Co' }];
    render(<OrganizationRedirectWrapper />);
    expect(screen.getByTestId('navigate')).toHaveAttribute('data-to', '/acme-corp/dashboard');
  });

  it('falls back to the first organization when none is stored', () => {
    organizationsState.organizations = [
      { id: 'org-1', name: 'Globex Inc' },
      { id: 'org-2', name: 'Other Co' },
    ];
    render(<OrganizationRedirectWrapper />);
    expect(screen.getByTestId('navigate')).toHaveAttribute('data-to', '/globex-inc/dashboard');
  });

  it('renders nothing when there is no stored organization and none to fall back on', () => {
    organizationsState.organizations = [];
    const { container } = render(<OrganizationRedirectWrapper />);
    expect(container).toBeEmptyDOMElement();
  });
});
