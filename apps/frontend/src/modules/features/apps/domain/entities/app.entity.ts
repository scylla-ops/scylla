export interface AppEntity {
  id: string;
  organizationId: string;
  name: string;
  isActive: boolean;
  createdAt: string;
  updatedAt: string;
}

/** Metadata only. Any enabled secret authenticates; disabling rejects it, revoking deletes it. */
export interface AppSecretEntity {
  id: string;
  appId: string;
  label: string;
  enabled: boolean;
  createdAt: string;
  updatedAt: string;
}
