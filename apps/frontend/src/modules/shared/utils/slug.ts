/**
 * Converts an organization name into a URL-friendly slug.
 * Example: "My Organization" → "my-organization"
 */
export const slugifyOrgName = (name: string): string =>
  name
    .toLowerCase()
    .trim()
    .replace(/[^a-z0-9]+/g, '-')
    .replace(/^-+|-+$/g, '');

