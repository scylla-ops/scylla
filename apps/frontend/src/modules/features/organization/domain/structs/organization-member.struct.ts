/**
 * Someone the organization has admitted, as shown in member lists.
 *
 * Membership is not stored: the backend derives it from the grants table —
 * anyone holding a grant on the organization, or on one of its projects, is a
 * member. There is therefore no "add member" or "remove member" RPC, and none
 * is missing: admitting someone is `CreateGrant` at the organization's scope
 * (the same call `AcceptInvitation` makes), and removing them is
 * `RevokeAllAccess` over that scope.
 *
 * A value object, not an entity: it carries no identity of its own beyond the
 * user it points at.
 */
export interface OrganizationMember {
  userId: string;
  username: string;
}
