export type CredentialKind = 'SSH_KEY' | 'TOKEN' | 'SECRET_TEXT' | 'LOGIN';

export type CredentialHealth = 'healthy' | 'warning' | 'idle';

export interface Credential {
  id: string;
  name: string;
  externalId: string;
  kind: CredentialKind;
  health: CredentialHealth;
  lastUsageLabel: string;
  createdAtLabel: string;
  expiresInDays?: number;
}

