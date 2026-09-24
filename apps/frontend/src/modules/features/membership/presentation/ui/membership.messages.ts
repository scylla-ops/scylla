import { msg, plural } from '@lingui/core/macro';

/** Keep the msgids (placeholder names included) or the French is lost. */
export const membershipMessages = {
  member: msg`Member`,
  members: msg`Members`,
  addAMember: msg`Add a member`,
  cancel: msg`Cancel`,
  add: msg`Add`,
  you: msg`You`,
  noRole: msg`No role`,
  loading: msg`Loading…`,
  noRoleGrantable: msg`No role can be granted here.`,
  held: msg`Held`,
  addRolePlaceholder: msg`+ role`,
  selectAUser: msg`Select a user`,
  nobodyListed: msg`Nobody is listed here yet.`,
  managedAtOrganization: msg`Managed at the organization level.`,
  revokeRole: (name: string) => msg`Revoke ${name}`,
  /** Keep `value` and the `0:` arm: they are part of the msgid. */
  roleCount: (value: number) =>
    msg`${plural(value, { 0: 'No role', one: '# role', other: '# roles' })}`,

  roleGranted: msg`Role granted`,
  roleRevoked: msg`Role revoked`,
  memberRemoved: (username: string, revoked: number) =>
    msg`${username} removed — ${revoked} grant(s) revoked`,
  memberAdded: (count: number) => msg`Member added with ${count} role(s)`,

  selectAnOrganization: msg`Select an organization to see its members.`,
  organizationBlurb: (name: string) =>
    msg`Belonging to “${name}” means holding a role in it. Granting someone their first role admits them; revoking every role they hold here, and on its projects, removes them.`,
  organizationNewDenied: msg`You don't have permission to admit members here.`,
  organizationEmpty: msg`Nobody belongs to this organization yet.`,
  reachedViaProject: msg`Reached via a project`,
  managedByOrgAdmin: msg`Managed by an organization administrator`,
  removeFromOrganization: msg`Remove from the organization`,
  addToOrganization: (name: string) => msg`Add a member to “${name}”`,
  addToOrganizationBody: msg`Pick someone and the roles they should hold in this organization.`,
  everyoneIsMember: msg`Everyone is already a member`,
  cannotBrowseDirectory: msg`You can't browse the user directory`,
  rolesToGrant: msg`Roles to grant`,
  confirmRemoveFromOrganization: (username: string, organization: string) =>
    msg`Remove ${username} from “${organization}”?`,
  confirmRemoveFromOrganizationBody: msg`Every role they hold on this organization and on its projects will be revoked. They will lose access immediately.`,

  projectBlurb: msg`Everyone with access to this project, whether granted here or through a role on the organization. Only project roles can be changed from this page.`,
  projectNewDenied: msg`You don't have permission to add members to this project.`,
  projectEmpty: msg`Nobody has access to this project yet.`,
  managedByProjectAdmin: msg`Managed by a project administrator`,
  removeFromProject: msg`Remove from the project`,
  inheritedHintVisible: msg`Roles badged “Organization” are inherited from the organization and are managed there.`,
  inheritedHintHidden: msg`Organization administrators also reach this project. Their roles are managed at the organization level and are not listed here.`,
  addToProject: msg`Add a member to this project`,
  addToProjectBody: msg`Only members of the organization can be added — a project role granted to anyone else is refused.`,
  everyOrgMemberHere: msg`Every member of the organization is already here`,
  cannotBrowseOrgMembers: msg`You can't browse the organization's members`,
  projectRolesToGrant: msg`Project roles to grant`,
  confirmRemoveFromProject: (username: string) => msg`Remove ${username} from this project?`,
  confirmRemoveFromProjectBody: msg`Every role they hold on this project will be revoked. Roles inherited from the organization are untouched — they are managed at the organization level.`,
};
