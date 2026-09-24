/** Metadata only: the value is write-only and never returned. */
export interface SecretEntity {
  id: string;
  projectId: string;
  name: string;
  description: string;
  createdAt: string;
  updatedAt: string;
}

/** `value` is sent once and never stored. */
export interface CreateSecretInput {
  projectId: string;
  name: string;
  value: string;
  description: string;
}
