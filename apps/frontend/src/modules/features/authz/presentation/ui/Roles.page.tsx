import { useState, type FormEvent } from 'react';
import { Scope } from '@/generated/permission.ts';
import type { Permission, Role } from '@/generated/permission.ts';
import { useRoles } from '@/modules/features/authz/presentation/hooks/use-roles.ts';
import {
  ALL_PERMISSIONS,
  ALL_SCOPES,
  errorMessage,
  permissionName,
  scopeName,
} from '@/modules/features/authz/presentation/utils/authz-labels.ts';

// Wireframe only: no styling beyond a scroll container and old-school table
// borders. Surfaces the dynamic-role CRUD (RoleService) end to end.
export const RolesPage = () => {
  const { roles, isLoading, isError, error, createRole, updateRole, deleteRole } = useRoles();

  const [editingId, setEditingId] = useState<string | null>(null);
  const [name, setName] = useState('');
  const [description, setDescription] = useState('');
  const [scope, setScope] = useState<Scope>(Scope.ORGANIZATION);
  const [fullControl, setFullControl] = useState(false);
  const [selected, setSelected] = useState<Permission[]>([]);

  const resetForm = () => {
    setEditingId(null);
    setName('');
    setDescription('');
    setScope(Scope.ORGANIZATION);
    setFullControl(false);
    setSelected([]);
  };

  const startEdit = (role: Role) => {
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
              <th>Builtin</th>
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
                <td>{r.builtin ? 'yes' : 'no'}</td>
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
              onChange={e => setScope(Number(e.target.value) as Scope)}
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
            {ALL_PERMISSIONS.map(p => (
              <div key={p}>
                <label>
                  <input
                    type='checkbox'
                    checked={selected.includes(p)}
                    onChange={() => togglePermission(p)}
                  />{' '}
                  {permissionName(p)}
                </label>
              </div>
            ))}
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
