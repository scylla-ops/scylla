import { describe, it, expect } from 'vitest';
import { humanizeRoleId } from './role-label';

describe('humanizeRoleId', () => {
  it('turns a kebab-case builtin id into a capitalized phrase', () => {
    expect(humanizeRoleId('organization-admin')).toBe('Organization admin');
  });

  it('also handles underscore-separated ids', () => {
    expect(humanizeRoleId('project_viewer')).toBe('Project viewer');
  });

  it('leaves an already-single-word id capitalized', () => {
    expect(humanizeRoleId('admin')).toBe('Admin');
  });

  it('falls back to the raw id (empty string) rather than throwing on an empty input', () => {
    expect(humanizeRoleId('')).toBe('');
  });

  it('falls back to the raw id when it is made entirely of separators', () => {
    expect(humanizeRoleId('---')).toBe('---');
  });
});
