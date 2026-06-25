/**
 * Domain entity for an App (machine principal / agent identity).
 */
export interface AppEntity {
  id: string;
  organizationId: string;
  name: string;
  isActive: boolean;
  createdAt: string;
  updatedAt: string;
}

/**
 * One named secret of an App. Metadata only — the plaintext is never returned
 * after creation. An App can hold several; authentication accepts any enabled
 * one. Disabling keeps it but rejects it at auth; revoking deletes it.
 */
export interface AppSecretEntity {
  id: string;
  appId: string;
  label: string;
  enabled: boolean;
  createdAt: string;
  updatedAt: string;
}
