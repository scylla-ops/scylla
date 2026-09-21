// @vitest-environment node
import { describe, it, expect } from 'vitest';
import { Permission, PermissionScope, PrincipalKind } from '@platform/authz';
import type { RoleEntity } from '../domain/entities/role.entity.ts';
import type { GrantEntity } from '../domain/entities/grant.entity.ts';
import { buildGrantEligibility } from './grant-eligibility.calculator.ts';

const ORG_ID = 'org-1';

const role = (overrides: Partial<RoleEntity> = {}): RoleEntity => ({
  id: 'role-1',
  name: 'Org member',
  description: '',
  scope: PermissionScope.ORGANIZATION,
  origin: { kind: 'builtin', key: 'organization-member' },
  access: { kind: 'restricted', permissions: [Permission.READ_ORGANIZATION] },
  ...overrides,
});

const grant = (overrides: Partial<GrantEntity> = {}): GrantEntity => ({
  id: 'grant-1',
  principal: { kind: PrincipalKind.USER, id: 'user-1' },
  roleId: 'role-1',
  scope: PermissionScope.ORGANIZATION,
  scopeId: ORG_ID,
  ...overrides,
});

const eligibilityOf = (grants: GrantEntity[], roles: RoleEntity[], orgId: string | null = ORG_ID) =>
  buildGrantEligibility(grants, roles, orgId)('user-1');

describe('buildGrantEligibility', () => {
  it('a user with no grant at all in the organization is "not-admitted"', () => {
    expect(eligibilityOf([], [])).toBe('not-admitted');
  });

  it('a user holding an organization role that confers READ_ORGANIZATION is "eligible"', () => {
    expect(
      eligibilityOf(
        [grant({ roleId: 'admin' })],
        [role({ id: 'admin', access: { kind: 'fullControl' } })],
      ),
    ).toBe('eligible');
  });

  it('admitted but without READ_ORGANIZATION on any held role is "cannot-see-projects"', () => {
    expect(
      eligibilityOf(
        [grant({ roleId: 'billing-only' })],
        [
          role({
            id: 'billing-only',
            access: { kind: 'restricted', permissions: [Permission.UPDATE_ORGANIZATION] },
          }),
        ],
      ),
    ).toBe('cannot-see-projects');
  });

  it('a grant bound to a DIFFERENT organization does not count', () => {
    expect(
      eligibilityOf([grant({ scopeId: 'org-2' })], [role({ access: { kind: 'fullControl' } })]),
    ).toBe('not-admitted');
  });

  it('a SYSTEM-scoped grant does not count as organization membership either', () => {
    expect(
      eligibilityOf(
        [grant({ scope: PermissionScope.SYSTEM, scopeId: '' })],
        [role({ access: { kind: 'fullControl' } })],
      ),
    ).toBe('not-admitted');
  });

  it('an app principal is not a user, whatever it holds', () => {
    expect(
      eligibilityOf(
        [grant({ principal: { kind: PrincipalKind.APP, id: 'user-1' } })],
        [role({ access: { kind: 'fullControl' } })],
      ),
    ).toBe('not-admitted');
  });

  it('nobody is admitted while the organization is still unknown', () => {
    expect(eligibilityOf([grant()], [role({ access: { kind: 'fullControl' } })], null)).toBe(
      'not-admitted',
    );
  });

  it('one qualifying role among several is enough', () => {
    expect(
      eligibilityOf(
        [grant({ id: 'g1', roleId: 'billing' }), grant({ id: 'g2', roleId: 'member' })],
        [
          role({ id: 'billing', access: { kind: 'restricted', permissions: [] } }),
          role({ id: 'member' }),
        ],
      ),
    ).toBe('eligible');
  });

  it('gives a role the catalog does not hold the benefit of the doubt', () => {
    // `roleConfers(undefined, …)` is true by design: a caller that cannot read
    // the catalog must not have every role treated as empty. The consequence
    // here is that an unknown role reads as admitting — the backend is still
    // the enforcer, and greying someone out on a guess is the worse failure.
    expect(eligibilityOf([grant({ roleId: 'ghost' })], [])).toBe('eligible');
  });
});
