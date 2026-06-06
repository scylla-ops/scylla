import { useState, type FormEvent } from 'react';
import { Permission, Scope } from '@/generated/permission.ts';
import { idValue } from '@core/infrastructure/grpc/wrappers.ts';
import { useGrants } from '@/modules/features/authz/presentation/hooks/use-grants.ts';
import { useRoles } from '@/modules/features/authz/presentation/hooks/use-roles.ts';
import {
  ALL_PERMISSIONS,
  ALL_SCOPES,
  errorMessage,
  permissionName,
  scopeName,
} from '@/modules/features/authz/presentation/utils/authz-labels.ts';

type TargetType = 'role' | 'permission';

// Wireframe only. Surfaces grants end to end (GrantService): grant a role OR a
// single permission to a user within a scope, list every grant, revoke.
export const GrantsPage = () => {
  const { grants, isLoading, isError, error, createGrant, revokeGrant } = useGrants();
  // Full role catalog (builtins + custom) so custom roles are now grantable; the
  // dropdown is filtered to roles declared on the selected scope, and the grant
  // references a role by its id. (listRoles needs manageRoles — fine here.)
  const { roles } = useRoles();

  const [userId, setUserId] = useState('');
  const [targetType, setTargetType] = useState<TargetType>('role');
  const [role, setRole] = useState('');
  const [permission, setPermission] = useState<Permission>(ALL_PERMISSIONS[0]);
  const [scope, setScope] = useState<Scope>(Scope.ORGANIZATION);
  const [scopeId, setScopeId] = useState('');

  const roleOptions = roles.filter(r => r.scope === scope);

  const submit = (e: FormEvent) => {
    e.preventDefault();
    const base = { userId, scope, scopeId };
    const input =
      targetType === 'permission' ? { ...base, role: '', permission } : { ...base, role };
    createGrant.mutate(input, {
      onSuccess: () => {
        setUserId('');
        setScopeId('');
      },
    });
  };

  const grantTarget = (g: (typeof grants)[number]) =>
    g.permission != null && g.permission !== Permission.PERMISSION_UNSPECIFIED
      ? `permission: ${permissionName(g.permission)}`
      : `role: ${roles.find(r => r.id === g.role)?.name ?? g.role}`;

  return (
    <div style={{ height: '100%', overflow: 'auto', padding: 16 }}>
      <h1>Grants</h1>
      <p>A grant binds a principal to a role or a single permission within a scope.</p>

      {isError && <p>Error loading grants: {errorMessage(error)}</p>}

      {isLoading ? (
        <p>Loading…</p>
      ) : (
        <table border={1} cellPadding={4}>
          <thead>
            <tr>
              <th>Grant id</th>
              <th>User</th>
              <th>Target</th>
              <th>Scope</th>
              <th>Scope id</th>
              <th>Actions</th>
            </tr>
          </thead>
          <tbody>
            {grants.length === 0 && (
              <tr>
                <td colSpan={6}>No grants.</td>
              </tr>
            )}
            {grants.map(g => (
              <tr key={g.id}>
                <td>{g.id}</td>
                <td>{idValue(g.userId)}</td>
                <td>{grantTarget(g)}</td>
                <td>{scopeName(g.scope)}</td>
                <td>{g.scopeId || '—'}</td>
                <td>
                  <button type='button' onClick={() => revokeGrant.mutate(g.id)}>
                    Revoke
                  </button>
                </td>
              </tr>
            ))}
          </tbody>
        </table>
      )}

      {revokeGrant.isError && <p>Revoke failed: {errorMessage(revokeGrant.error)}</p>}

      <hr />

      <h2>Create grant</h2>
      <form onSubmit={submit}>
        <div>
          <label>
            User id <input value={userId} onChange={e => setUserId(e.target.value)} required />
          </label>
        </div>

        <div>
          <label>
            <input
              type='radio'
              name='targetType'
              checked={targetType === 'role'}
              onChange={() => setTargetType('role')}
            />{' '}
            Role
          </label>{' '}
          <label>
            <input
              type='radio'
              name='targetType'
              checked={targetType === 'permission'}
              onChange={() => setTargetType('permission')}
            />{' '}
            Single permission
          </label>
        </div>

        {targetType === 'role' ? (
          <div>
            <label>
              Role{' '}
              <select value={role} onChange={e => setRole(e.target.value)} required>
                <option value=''>— select a role —</option>
                {roleOptions.map(r => (
                  <option key={r.id} value={r.id}>
                    {r.name}
                    {r.builtin ? '' : ' · custom'}
                  </option>
                ))}
              </select>
              {roleOptions.length === 0 && <small> no role on this scope</small>}
            </label>
          </div>
        ) : (
          <div>
            <label>
              Permission{' '}
              <select
                value={permission}
                onChange={e => setPermission(Number(e.target.value) as Permission)}
              >
                {ALL_PERMISSIONS.map(p => (
                  <option key={p} value={p}>
                    {permissionName(p)}
                  </option>
                ))}
              </select>
            </label>
          </div>
        )}

        <div>
          <label>
            Scope{' '}
            <select
              value={scope}
              onChange={e => {
                setScope(Number(e.target.value) as Scope);
                setRole('');
              }}
            >
              {ALL_SCOPES.map(s => (
                <option key={s} value={s}>
                  {scopeName(s)}
                </option>
              ))}
            </select>
          </label>
        </div>

        <div>
          <label>
            Scope id{' '}
            <input
              value={scopeId}
              onChange={e => setScopeId(e.target.value)}
              placeholder='org/project id — ignored for SYSTEM'
            />
          </label>
        </div>

        <div>
          <button type='submit' disabled={createGrant.isPending}>
            Create grant
          </button>
        </div>
      </form>

      {createGrant.isError && <p>Grant failed: {errorMessage(createGrant.error)}</p>}
    </div>
  );
};
