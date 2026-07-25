/**
 * A user's membership in an organization, as shown in member lists.
 * Identity-bearing: a member is a user (by `userId`) within an organization.
 */
export interface OrganizationMemberEntity {
  userId: string;
  username: string;
}
