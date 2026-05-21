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
