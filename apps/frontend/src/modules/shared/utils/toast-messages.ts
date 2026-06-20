import { msg } from '@lingui/core/macro';
import type { MessageDescriptor } from '@lingui/core';

/**
 * Centralized toast message descriptors.
 *
 * Usage:
 *   import { useLingui } from '@lingui/react/macro';
 *   import { ToastMessages } from '@shared/utils/toast-messages';
 *
 *   const { _ } = useLingui();
 *   toast.success(_(ToastMessages.USER_CREATE));
 *
 * Messages are statically extractable by the Lingui CLI and land in
 * src/locales/{locale}/messages (shared catalog).
 */
export const ToastMessages = {
  // Users
  USER_CREATE: msg`User created`,
  USER_DELETE: msg`User deleted`,
  USER_UPDATE: msg`User information updated`,
  USER_DELETE_OWN_ACCOUNT_ERROR: msg`You cannot delete your own account.`,
  USER_CREDENTIALS_REQUIRED_ERROR: msg`Username and password are required.`,

  // Organizations
  ORGANIZATION_CREATE: msg`Organization created`,
  ORGANIZATION_DELETE: msg`Organization deleted`,
  ORGANIZATION_UPDATE: msg`Organization updated`,

  // Projects
  PROJECT_CREATE: msg`Project created`,
  PROJECT_DELETE: msg`Project deleted`,
  PROJECT_UPDATE: msg`Project updated`,
  PROJECT_NAME_REQUIRED_ERROR: msg`Project name is required and you must select an organization.`,

  // Pipelines
  PIPELINE_CREATE: msg`Pipeline created`,
  PIPELINE_DELETE: msg`Pipeline deleted`,
  PIPELINE_UPDATE: msg`Pipeline edited`,
  PIPELINE_RUN: msg`Pipeline ran`,
  PIPELINE_JOB_QUEUED_WARNING: msg`Job queued — no agent connected`,

  // Agents
  AGENT_ID_COPIED: msg`Agent id copied`,
  AGENT_ID_COPY_ERROR: msg`Could not copy — select the id manually`,

  // Apps
  APP_ID_COPIED: msg`App id copied`,

  // Secrets
  SECRET_CREATE: msg`Secret created`,
  SECRET_DELETE: msg`Secret deleted`,

  // Shared
  COPIED: msg`Copied`,
} as const satisfies Record<string, MessageDescriptor>;
