import { msg, plural } from '@lingui/core/macro';

/**
 * Every string the secrets screen shows.
 *
 * `lingui extract` does not read `.svelte`, so a message declared inside a
 * component would vanish from the catalogs without failing a single gate. The
 * ids below are byte-identical to the ones the React components carried, which
 * is what keeps the French translations attached — **including placeholder
 * names**, which are part of the msgid.
 */
export const secretMessages = {
  // Header
  secret: msg`Secret`,
  secrets: msg`Secrets`,
  newSecret: msg`New secret`,
  createDenied: msg`You don't have permission to create secrets.`,
  deleteDenied: msg`You don't have permission to delete secrets.`,

  // Create dialog
  createSecret: msg`Create secret`,

  // Table
  name: msg`Name`,
  description: msg`Description`,
  created: msg`Created`,
  actions: msg`Actions`,
  /**
   * New, and deliberately: the React trash button was an icon with no
   * accessible name, so nothing could find it but a CSS selector. The label is
   * `sr-only` — the control looks the same.
   */
  deleteSecret: msg`Delete secret`,

  // Health overview
  rotationPolicy: msg`Rotation Policy`,
  rotationOverdue: (warningCount: number) =>
    msg`${plural(warningCount, {
      one: '# credential overdue for rotation based on your enterprise policy (90 days).',
      other: '# credentials overdue for rotation based on your enterprise policy (90 days).',
    })}`,
  reviewPolicy: msg`Review Policy`,
  vaultHealth: msg`Vault Health`,
  uptimeStatus: msg`Uptime status`,
  lastSync: msg`Last sync: 2 min ago`,
  auditLogging: msg`Audit Logging`,
  accessAttempts: msg`Access attempts (24h)`,
  unauthorizedAttempts: msg`Unauthorized attempts`,
  viewAuditLogs: msg`View Audit Logs`,

  // Pagination
  /**
   * The React original wrapped each number in a `<span class='font-semibold'>`
   * through `<Trans>`, which encoded them as `<0>`/`<1>`/`<2>` in the msgid.
   * A `t()` call returns a string and cannot carry element slots, so the
   * numbers are plain placeholders here — a new msgid, whose French form is
   * carried over by hand in `locales/fr/messages.po`.
   */
  showingRange: (firstItem: number, lastItem: number, totalItems: number) =>
    msg`Showing ${firstItem}-${lastItem} of ${totalItems} credentials`,
  previousPage: msg`Prev`,
  nextPage: msg`Next`,
};
