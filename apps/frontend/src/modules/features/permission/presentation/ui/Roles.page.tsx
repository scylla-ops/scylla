import { useState, type FormEvent } from 'react';
import { Permission, PermissionScope } from '@/modules/features/permission/domain/models/permission.model.ts';
import type { RoleEntity } from '@/modules/features/permission/domain/entities/role.entity.ts';
import { useRoles } from '@/modules/features/permission/presentation/hooks/use-roles.ts';
import { useAuthzVocabulary } from '@/modules/features/permission/presentation/hooks/use-authz-vocabulary.ts';
import {
  ALL_PERMISSIONS,
  ALL_SCOPES,
  errorMessage,
  permissionName,
  scopeName,
} from '@/modules/features/permission/presentation/utils/authz-labels.ts';

// Wireframe only: no styling beyond a scroll container and old-school table
// borders. Surfaces the dynamic-role CRUD (RoleService) end to end.
export const RolesPage = () => {
  const { roles, isLoading, isError, error, createRole, updateRole, deleteRole } = useRoles();
  // Coherence oracle: which permissions actually do something at a given scope.
  const { coherentAtScope } = useAuthzVocabulary();

  const [editingId, setEditingId] = useState<string | null>(null);
  const [name, setName] = useState('');
  const [description, setDescription] = useState('');
  const [scope, setScope] = useState<PermissionScope>(PermissionScope.ORGANIZATION);
  const [fullControl, setFullControl] = useState(false);
  const [selected, setSelected] = useState<Permission[]>([]);

  const resetForm = () => {
    setEditingId(null);
    setName('');
    setDescription('');
    setScope(PermissionScope.ORGANIZATION);
    setFullControl(false);
    setSelected([]);
  };

  const startEdit = (role: RoleEntity) => {
    setEditingId(role.id);
    setName(role.name);
    setDescription(role.description);
    setScope(role.scope);
    setFullControl(role.fullControl);
    setSelected(role.permissions);
  };

  const togglePermission = (p: Permission) =>
    setSelected(prev => (prev.includes(p) ? prev.filter(x => x !== p) : [...prev, p]));

  const submit = (e: FormEvent) => {
    e.preventDefault();
    const permissions = fullControl ? [] : selected;
    if (editingId) {
      updateRole.mutate(
        { id: editingId, name, description, fullControl, permissions },
        { onSuccess: resetForm },
      );
    } else {
      createRole.mutate(
        { name, description, scope, fullControl, permissions },
        { onSuccess: resetForm },
      );
    }
  };

  const pending = createRole.isPending || updateRole.isPending;

  return (
    <div style={{ height: '100%', overflow: 'auto', padding: 16 }}>
      <h1>Roles</h1>
      <p>
        Dynamic role catalog — a role is an editable bundle of permissions bound to a scope kind.
      </p>

      {isError && <p>Error loading roles: {errorMessage(error)}</p>}

      {isLoading ? (
        <p>Loading…</p>
      ) : (
        <table border={1} cellPadding={4}>
          <thead>
            <tr>
              <th>Name / id</th>
              <th>Scope</th>
              <th>Origin</th>
              <th>Permissions</th>
              <th>Actions</th>
            </tr>
          </thead>
          <tbody>
            {roles.length === 0 && (
              <tr>
                <td colSpan={5}>No roles.</td>
              </tr>
            )}
            {roles.map(r => (
              <tr key={r.id}>
                <td>
                  {r.name}
                  <br />
                  <small>{r.id}</small>
                </td>
                <td>{scopeName(r.scope)}</td>
                <td>{r.builtin ? 'System' : 'Custom'}</td>
                <td>
                  {r.fullControl
                    ? '* (full control)'
                    : r.permissions.map(permissionName).join(', ') || '—'}
                </td>
                <td>
                  <button type='button' onClick={() => startEdit(r)}>
                    Edit
                  </button>{' '}
                  <button
                    type='button'
                    disabled={r.builtin}
                    onClick={() => deleteRole.mutate(r.id)}
                  >
                    Delete
                  </button>
                </td>
              </tr>
            ))}
          </tbody>
        </table>
      )}

      {deleteRole.isError && <p>Delete failed: {errorMessage(deleteRole.error)}</p>}

      <hr />

      <h2>{editingId ? `Edit role (${editingId})` : 'Create role'}</h2>
      <form onSubmit={submit}>
        <div>
          <label>
            Name <input value={name} onChange={e => setName(e.target.value)} required />
          </label>
        </div>
        <div>
          <label>
            Description <input value={description} onChange={e => setDescription(e.target.value)} />
          </label>
        </div>
        <div>
          <label>
            Scope{' '}
            <select
              value={scope}
              disabled={!!editingId}
              onChange={e => {
                const next = Number(e.target.value) as PermissionScope;
                setScope(next);
                // Drop any now-incoherent picks so the role can't carry dead perms.
                setSelected(prev => prev.filter(p => coherentAtScope(p, next)));
              }}
            >
              {ALL_SCOPES.map(s => (
                <option key={s} value={s}>
                  {scopeName(s)}
                </option>
              ))}
            </select>
          </label>
          {editingId && <small> (scope is immutable)</small>}
        </div>
        <div>
          <label>
            <input
              type='checkbox'
              checked={fullControl}
              onChange={e => setFullControl(e.target.checked)}
            />{' '}
            Full control (confers every permission within the scope)
          </label>
        </div>

        {!fullControl && (
          <fieldset>
            <legend>Permissions ({selected.length} selected)</legend>
            <p>
              <small>
                Greyed-out permissions are not coherent with the {scopeName(scope)} scope (their
                target resource lives outside it), so they are disabled.
              </small>
            </p>
            {ALL_PERMISSIONS.map(p => {
              const ok = coherentAtScope(p, scope);
              return (
                <div key={p}>
                  <label>
                    <input
                      type='checkbox'
                      disabled={!ok}
                      checked={ok && selected.includes(p)}
                      onChange={() => togglePermission(p)}
                    />{' '}
                    {permissionName(p)}
                    {ok ? '' : ' — n/a at this scope'}
                  </label>
                </div>
              );
            })}
          </fieldset>
        )}

        <div>
          <button type='submit' disabled={pending}>
            {editingId ? 'Save' : 'Create'}
          </button>{' '}
          {editingId && (
            <button type='button' onClick={resetForm}>
              Cancel
            </button>
          )}
        </div>
      </form>

      {(createRole.isError || updateRole.isError) && (
        <p>Save failed: {errorMessage(createRole.error ?? updateRole.error)}</p>
      )}
    </div>
  );
};
