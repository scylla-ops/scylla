import { msg } from '@lingui/core/macro';

/**
 * Every string the roles screens show.
 *
 * `lingui extract` does not read `.svelte`, so a message declared inside a
 * component would vanish from the catalogs without failing a single gate. The
 * ids below are byte-identical to the ones the React components carried —
 * placeholder names included, since those are part of the msgid.
 *
 * Three of them interpolate a positional `{0}`, because the `<Trans>` they came
 * from was given an expression rather than an identifier. `String(…)` /
 * `Number(…)` keeps them expressions here, which is what keeps the msgid — and
 * therefore the French — attached.
 *
 * Permission and scope names are **not** here: they live in
 * `presentation/utils/permission-mapping.ts`, beside the catalog that defines
 * them, so a new backend permission needs one entry rather than two.
 */
export const rolesMessages = {
  // List page
  role: msg`Role`,
  roles: msg`Roles`,
  createRole: msg`Create role`,
  noRoles: msg`No roles yet. Create one to get started.`,
  selectARole: msg`Select a role to see its permissions and members.`,
  noDescription: msg`No description`,
  memberCount: (memberCount: number) => msg`${memberCount} members`,

  // Detail header
  builtin: msg`Built-in`,
  custom: msg`Custom`,
  unknownOrigin: msg({ context: 'feminine', message: 'Unknown' }),
  edit: msg`Edit`,
  editDenied: msg`You don't have permission to edit roles.`,
  fullControl: msg`Full control`,

  // Detail permissions
  permissions: msg`Permissions`,
  grantsFullControl: msg`Grants full control over its scope.`,
  noPermissions: msg`No permissions.`,
  unknownAccess: msg`Unknown access.`,

  // Detail grants
  grants: msg`Grants`,
  noGrants: msg`No one holds this role yet.`,
  remove: msg`Remove`,
  revokeDenied: msg`You don't have permission to revoke grants.`,

  // Grant dialog
  addGrant: msg`Add grant`,
  grantDenied: msg`You don't have permission to grant this role.`,
  grantTitle: (roleName: string) => msg`Grant “${String(roleName)}”`,
  grantSubtitle: msg`Choose who receives this role and where it applies.`,
  systemWide: msg`This role grants access across the whole system.`,
  user: msg`User`,
  selectAUser: msg`Select a user`,
  pickAnOrganizationFirst: msg`Pick an organization first`,
  notAMember: msg`not a member of this organization`,
  cannotSeeProjects: msg`can't see this organization's projects`,
  organization: msg`Organization`,
  organizations: msg`Organizations`,
  pickAnOrganization: msg`Pick an organization`,
  noOrganizations: msg`No organizations available.`,
  projects: msg`Projects`,
  noProjects: msg`No projects in this organization.`,
  nobodyEligible: msg`Nobody can receive a project grant here yet. Admit someone to the organization first — from the organization switcher, under “Members” — and they will show up here.`,
  loading: msg`Loading…`,
  granted: msg`Granted`,
  selectedCount: (count: number) => msg`Selected (${Number(count)})`,
  cancel: msg`Cancel`,
  createGrant: msg`Create grant`,
  grantCreated: msg`Grant created`,
  grantsCreated: (count: number) => msg`${Number(count)} grants created`,
  grantFailed: msg`Failed to create grant`,

  // Role form
  editRoleTitle: msg`Edit role`,
  roleFormSubtitle: msg`Define what this role is called and what it can do.`,
  name: msg`Name`,
  namePlaceholder: msg`e.g., project-viewer`,
  description: msg`Description`,
  descriptionPlaceholder: msg`What is this role for?`,
  scope: msg`Scope`,
  scopeIsFixed: msg`Scope cannot be changed after creation.`,
  access: msg`Access`,
  restricted: msg`Restricted permissions`,
  saveChanges: msg`Save changes`,

  // Permission picker
  conferredCount: (conferredCount: number) => msg`${conferredCount} selected`,
  always: msg`Always`,
  /**
   * New, and deliberately: the expand/collapse control of the permission tree
   * had no accessible name at all in React — a `+` glyph in a button — so
   * nothing but a CSS selector could reach it, and a screen reader announced
   * "button".
   */
  showSubPermissions: (label: string) => msg`Show ${label} sub-permissions`,
  hideSubPermissions: (label: string) => msg`Hide ${label} sub-permissions`,
  alwaysGrantedNote: msg`Holding a role in an organization is what belonging to it means, so every organization role carries it. An organization role applies to every project of the organization.`,
  preservedNote: (preservedCount: number) =>
    msg`This role also holds ${preservedCount} permission(s) not managed here. They are kept unchanged.`,
};
