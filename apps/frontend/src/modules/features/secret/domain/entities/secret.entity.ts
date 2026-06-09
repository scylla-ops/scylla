/**
 * Domain model for a project-scoped secret. Metadata only — the plaintext value
 * is write-only and is never returned by the API, so it never lives here.
 */
export interface SecretEntity {
  id: string;
  projectId: string;
  name: string;
  description: string;
  createdAt: string;
  updatedAt: string;
}

/** Fields needed to create a secret. `value` is write-only (sent once, never stored). */
export interface CreateSecretInput {
  projectId: string;
  name: string;
  value: string;
  description: string;
}
