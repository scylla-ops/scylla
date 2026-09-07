/**
 * A readable name for a role known only by its id.
 *
 * Builtin role ids are stable kebab-case keys (`organization-admin`), so the
 * id is already the name with the punctuation in the wrong place. Custom roles
 * carry an opaque id and a real name in the catalog — this is the fallback for
 * a caller that may not read the catalog, and it is honest: showing the raw id
 * beats showing "Unknown role".
 */
export const humanizeRoleId = (roleId: string): string => {
  const words = roleId.replace(/[-_]/g, ' ').trim();
  if (words === '') return roleId;
  return words.charAt(0).toUpperCase() + words.slice(1);
};
