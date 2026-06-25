import type {
  AppEntity,
  AppSecretEntity,
} from '@/modules/features/apps/domain/entities/app.entity.ts';

/**
 * Result of creating an App. The secret is returned exactly once, at creation
 * time, and is never retrievable again.
 */
export interface CreatedApp {
  app: AppEntity;
  secret: string;
}

/**
 * Result of creating/regenerating a secret. The plaintext is shown exactly once.
 */
export interface CreatedAppSecret {
  credential: AppSecretEntity;
  secret: string;
}
