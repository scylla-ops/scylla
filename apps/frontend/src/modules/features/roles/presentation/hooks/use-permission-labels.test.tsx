import { describe, it, expect } from 'vitest';
import type { ReactNode } from 'react';
import { renderHook } from '@testing-library/react';
import { I18nProvider } from '@lingui/react';
import { i18n } from '@lingui/core';
import { Permission, PermissionScope } from '@platform/authz';
import { usePermissionLabels } from './use-permission-labels';

const Wrapper = ({ children }: { children: ReactNode }) => (
  <I18nProvider i18n={i18n}>{children}</I18nProvider>
);

describe('usePermissionLabels', () => {
  it('permissionLabel resolves a cataloged permission to its label', () => {
    const { result } = renderHook(() => usePermissionLabels(), { wrapper: Wrapper });
    expect(result.current.permissionLabel(Permission.CREATE_PROJECT)).toBe('Create projects');
  });

  it('permissionLabel falls back to the humanized enum key for a permission outside the catalog', () => {
    const { result } = renderHook(() => usePermissionLabels(), { wrapper: Wrapper });
    expect(result.current.permissionLabel(Permission.UNSPECIFIED)).toBe('Unspecified');
  });

  it('switches to the broadLabel when the role\'s scope is broader than the permission\'s own', () => {
    const { result } = renderHook(() => usePermissionLabels(), { wrapper: Wrapper });
    // READ_PROJECT's own scope is PROJECT; read from an ORGANIZATION role it
    // means "every project", so the broad wording applies.
    expect(result.current.permissionLabel(Permission.READ_PROJECT, PermissionScope.ORGANIZATION)).toBe(
      'Open any project',
    );
  });

  it('uses the plain label when the role scope matches the permission\'s own scope exactly', () => {
    const { result } = renderHook(() => usePermissionLabels(), { wrapper: Wrapper });
    expect(result.current.permissionLabel(Permission.READ_PROJECT, PermissionScope.PROJECT)).toBe(
      'Member of the project',
    );
  });

  it('uses the plain label when no roleScope is given at all', () => {
    const { result } = renderHook(() => usePermissionLabels(), { wrapper: Wrapper });
    expect(result.current.permissionLabel(Permission.READ_PROJECT)).toBe('Member of the project');
  });

  it('falls back to the plain label when a cataloged permission has no broadLabel of its own', () => {
    const { result } = renderHook(() => usePermissionLabels(), { wrapper: Wrapper });
    // CREATE_PROJECT has no broadLabel entry - a broader role scope should not throw.
    expect(result.current.permissionLabel(Permission.CREATE_PROJECT, PermissionScope.SYSTEM)).toBe(
      'Create projects',
    );
  });

  it('scopeLabel resolves every PermissionScope, including UNSPECIFIED', () => {
    const { result } = renderHook(() => usePermissionLabels(), { wrapper: Wrapper });
    expect(result.current.scopeLabel(PermissionScope.SYSTEM)).toBe('System');
    expect(result.current.scopeLabel(PermissionScope.ORGANIZATION)).toBe('Organization');
    expect(result.current.scopeLabel(PermissionScope.PROJECT)).toBe('Project');
    expect(result.current.scopeLabel(PermissionScope.UNSPECIFIED)).toBe('Unknown');
  });
});
