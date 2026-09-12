import { describe, it, expect, vi } from 'vitest';
import { PermissionScope } from '@platform/authz';
import { ScyllaResult, ScyllaError } from '@shared/utils/scylla-result.ts';
import { UpdateRoleUseCase } from './update-role.use-case';
import type { PermissionRepository } from '@/modules/features/roles/domain/repository/permission.repository.ts';
import type { RoleEntity } from '@/modules/features/roles/domain/entities/role.entity.ts';

const role = (overrides: Partial<RoleEntity> = {}): RoleEntity => ({
  id: 'role-1',
  name: 'CI runner',
  description: 'runs pipelines',
  scope: PermissionScope.PROJECT,
  origin: { kind: 'custom' },
  access: { kind: 'fullControl' },
  ...overrides,
});

describe('UpdateRoleUseCase', () => {
  it('reads the role, applies the changes, and saves the merged result — not the raw input', async () => {
    const getRole = vi.fn().mockResolvedValue(ScyllaResult.success(role()));
    const updateRole = vi.fn().mockImplementation((r: RoleEntity) => Promise.resolve(ScyllaResult.success(r)));
    const repository = { getRole, updateRole } as unknown as PermissionRepository;
    const useCase = new UpdateRoleUseCase(repository);

    const result = await useCase.execute({ id: 'role-1', name: 'Renamed' });

    expect(getRole).toHaveBeenCalledWith('role-1');
    expect(updateRole).toHaveBeenCalledWith(
      expect.objectContaining({ id: 'role-1', name: 'Renamed', description: 'runs pipelines' }),
    );
    expect(result.unwrap().name).toBe('Renamed');
  });

  it('never calls updateRole when getRole fails, and propagates its error', async () => {
    const error = new ScyllaError('boom', { cause: { code: 'NOT_FOUND' } });
    const getRole = vi.fn().mockResolvedValue(ScyllaResult.error(error));
    const updateRole = vi.fn();
    const repository = { getRole, updateRole } as unknown as PermissionRepository;
    const useCase = new UpdateRoleUseCase(repository);

    const result = await useCase.execute({ id: 'missing', name: 'x' });

    expect(updateRole).not.toHaveBeenCalled();
    expect(() => result.unwrap()).toThrow(error);
  });

  it('never calls the repository\'s updateRole when the merge itself is invalid (empty name)', async () => {
    const getRole = vi.fn().mockResolvedValue(ScyllaResult.success(role()));
    const updateRole = vi.fn();
    const repository = { getRole, updateRole } as unknown as PermissionRepository;
    const useCase = new UpdateRoleUseCase(repository);

    const result = await useCase.execute({ id: 'role-1', name: '' });

    expect(updateRole).not.toHaveBeenCalled();
    expect(result.fold({ onSuccess: () => 'ok', onError: e => e.message })).toBe(
      'Error mapping value',
    );
  });

  it('propagates a failure from the save call itself', async () => {
    const error = new ScyllaError('conflict', { cause: { code: 'ALREADY_EXISTS' } });
    const getRole = vi.fn().mockResolvedValue(ScyllaResult.success(role()));
    const updateRole = vi.fn().mockResolvedValue(ScyllaResult.error(error));
    const repository = { getRole, updateRole } as unknown as PermissionRepository;
    const useCase = new UpdateRoleUseCase(repository);

    const result = await useCase.execute({ id: 'role-1', name: 'renamed' });

    expect(() => result.unwrap()).toThrow(error);
  });
});
