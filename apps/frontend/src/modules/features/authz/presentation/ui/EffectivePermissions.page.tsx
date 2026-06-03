import { useState, type FormEvent } from 'react';
import { PrincipalKind } from '@/generated/permission.ts';
import { useEffectivePermissions } from '@/modules/features/authz/presentation/hooks/use-effective-permissions.ts';
import {
  ALL_PRINCIPAL_KINDS,
  errorMessage,
  permissionName,
  principalKindName,
  scopeName,
} from '@/modules/features/authz/presentation/utils/authz-labels.ts';

// Wireframe only. Introspection: "what can this principal do" — roles expanded
// plus direct permission grants, grouped by the scope each grant binds to
// (RoleService.GetEffectivePermissions).
export const EffectivePermissionsPage = () => {
  const lookup = useEffectivePermissions();
  const [principalKind, setPrincipalKind] = useState<PrincipalKind>(PrincipalKind.USER);
  const [principalId, setPrincipalId] = useState('');

  const submit = (e: FormEvent) => {
    e.preventDefault();
    lookup.mutate({ principalKind, principalId });
  };

  return (
    <div style={{ height: '100%', overflow: 'auto', padding: 16 }}>
      <h1>Effective permissions</h1>
      <p>What a principal can do — roles expanded + direct permission grants, grouped by scope.</p>

      <form onSubmit={submit}>
        <div>
          <label>
            Principal kind{' '}
            <select
              value={principalKind}
              onChange={e => setPrincipalKind(Number(e.target.value) as PrincipalKind)}
            >
              {ALL_PRINCIPAL_KINDS.map(k => (
                <option key={k} value={k}>
                  {principalKindName(k)}
                </option>
              ))}
            </select>
          </label>
        </div>
        <div>
          <label>
            Principal id{' '}
            <input
              value={principalId}
              onChange={e => setPrincipalId(e.target.value)}
              required
              placeholder='user or app id'
            />
          </label>
        </div>
        <div>
          <button type='submit' disabled={lookup.isPending}>
            Look up
          </button>
        </div>
      </form>

      {lookup.isError && <p>Lookup failed: {errorMessage(lookup.error)}</p>}

      {lookup.data &&
        (lookup.data.scopes.length === 0 ? (
          <p>No effective permissions.</p>
        ) : (
          lookup.data.scopes.map((s, i) => (
            <div key={`${s.scope}-${s.scopeId}-${i}`}>
              <h3>
                {scopeName(s.scope)}
                {s.scopeId ? ` (${s.scopeId})` : ''}
              </h3>
              {s.fullControl ? (
                <p>* full control (every permission in this scope)</p>
              ) : (
                <ul>
                  {s.permissions.map(p => (
                    <li key={p}>{permissionName(p)}</li>
                  ))}
                </ul>
              )}
            </div>
          ))
        ))}
    </div>
  );
};
