import { describe, it, expect, beforeEach } from 'vitest';
import { render, screen } from '@testing-library/svelte';
import { Permission, PermissionScope, permissionsStore } from '@platform/authz';
import CanFixture from './Can.fixture.svelte';

beforeEach(() => {
  permissionsStore.setState({ permissions: null });
});

const grantEverything = () =>
  permissionsStore.setState({
    permissions: {
      scopes: [{ scope: PermissionScope.SYSTEM, scopeId: '', access: { kind: 'fullControl' } }],
    },
  });

describe('Can', () => {
  it('renders the children when the user holds the permission', () => {
    grantEverything();
    render(CanFixture, { permission: Permission.READ_PROJECT });

    expect(screen.getByText('visible content')).toBeInTheDocument();
  });

  it('renders nothing by default when the user lacks the permission', () => {
    permissionsStore.setState({ permissions: { scopes: [] } });
    const { container } = render(CanFixture, { permission: Permission.READ_PROJECT });

    expect(screen.queryByText('visible content')).not.toBeInTheDocument();
    expect(container.textContent?.trim()).toBe('');
  });

  it('renders the fallback instead, when given one', () => {
    permissionsStore.setState({ permissions: { scopes: [] } });
    render(CanFixture, { permission: Permission.READ_PROJECT, withFallback: true });

    expect(screen.getByText('fallback content')).toBeInTheDocument();
    expect(screen.queryByText('visible content')).not.toBeInTheDocument();
  });

  it('denies while the permissions are still unknown', () => {
    render(CanFixture, { permission: Permission.READ_PROJECT, withFallback: true });

    expect(screen.getByText('fallback content')).toBeInTheDocument();
  });

  it('shows the children when the permissions arrive after the first render', async () => {
    render(CanFixture, { permission: Permission.READ_PROJECT });
    grantEverything();

    expect(await screen.findByText('visible content')).toBeInTheDocument();
  });
});
