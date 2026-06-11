/**
 * Domain model for an App (machine principal / agent identity).
 */
export interface App {
  id: string;
  organizationId: string;
  name: string;
  isActive: boolean;
  createdAt: string;
  updatedAt: string;
}

/**
 * Result of creating an App. The secret is returned exactly once, at creation
 * time, and is never retrievable again.
 */
export interface CreatedApp {
  app: App;
  secret: string;
}

/**
 * One named secret of an App. Metadata only — the plaintext is never returned
 * after creation. An App can hold several; authentication accepts any enabled
 * one. Disabling keeps it but rejects it at auth; revoking deletes it.
 */
export interface AppSecret {
  id: string;
  appId: string;
  label: string;
  enabled: boolean;
  createdAt: string;
  updatedAt: string;
}

/**
 * Result of creating/regenerating a secret. The plaintext is shown exactly once.
 */
export interface CreatedAppSecret {
  credential: AppSecret;
  secret: string;
}
