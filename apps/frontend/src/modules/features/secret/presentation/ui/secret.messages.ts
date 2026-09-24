import { msg, plural } from '@lingui/core/macro';

/** Keep the msgids (placeholder names included) or the French is lost. */
export const secretMessages = {
  secret: msg`Secret`,
  secrets: msg`Secrets`,
  newSecret: msg`New secret`,
  createDenied: msg`You don't have permission to create secrets.`,
  deleteDenied: msg`You don't have permission to delete secrets.`,

  createSecret: msg`Create secret`,

  name: msg`Name`,
  description: msg`Description`,
  created: msg`Created`,
  actions: msg`Actions`,
  deleteSecret: msg`Delete secret`,

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

  /** Plain placeholders (a `t()` string has no element slots): the French was carried by hand. */
  showingRange: (firstItem: number, lastItem: number, totalItems: number) =>
    msg`Showing ${firstItem}-${lastItem} of ${totalItems} credentials`,
  previousPage: msg`Prev`,
  nextPage: msg`Next`,
};
