import { describe, it, expect, beforeEach } from 'vitest';
import { render, screen } from '@testing-library/react';
import { Can } from './Can';
import { usePermissionsStore, PermissionScope, Permission } from '@platform/authz';

beforeEach(() => {
  usePermissionsStore.setState({ permissions: null });
});

describe('Can', () => {
  it('renders children when the user holds the permission', () => {
    usePermissionsStore.setState({
      permissions: { scopes: [{ scope: PermissionScope.SYSTEM, scopeId: '', access: { kind: 'fullControl' } }] },
    });

    render(
      <Can permission={Permission.READ_PROJECT}>
        <span>visible content</span>
      </Can>,
    );

    expect(screen.getByText('visible content')).toBeInTheDocument();
  });

  it('renders nothing (not a fallback) by default when the user lacks the permission', () => {
    usePermissionsStore.setState({ permissions: { scopes: [] } });

    const { container } = render(
      <Can permission={Permission.READ_PROJECT}>
        <span>visible content</span>
      </Can>,
    );

    expect(screen.queryByText('visible content')).not.toBeInTheDocument();
    expect(container).toBeEmptyDOMElement();
  });

  it('renders the given fallback instead, when provided', () => {
    usePermissionsStore.setState({ permissions: { scopes: [] } });

    render(
      <Can permission={Permission.READ_PROJECT} fallback={<span>fallback content</span>}>
        <span>visible content</span>
      </Can>,
    );

    expect(screen.getByText('fallback content')).toBeInTheDocument();
    expect(screen.queryByText('visible content')).not.toBeInTheDocument();
  });

  it('denies (renders the fallback) while permissions are still unknown', () => {
    // permissions: null (the default) -> ready is false, can() denies
    render(
      <Can permission={Permission.READ_PROJECT} fallback={<span>fallback content</span>}>
        <span>visible content</span>
      </Can>,
    );

    expect(screen.getByText('fallback content')).toBeInTheDocument();
  });
});
