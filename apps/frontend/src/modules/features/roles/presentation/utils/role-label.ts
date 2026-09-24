/** Builtin ids are kebab-case keys (`organization-admin`): the fallback when the catalog is not readable. */
export const humanizeRoleId = (roleId: string): string => {
  const words = roleId.replace(/[-_]/g, ' ').trim();
  if (words === '') return roleId;
  return words.charAt(0).toUpperCase() + words.slice(1);
};
