import type {
  AppEntity,
  AppSecretEntity,
} from '@/modules/features/apps/domain/entities/app.entity.ts';

/** The secret is returned once, at creation, and never again. */
export interface CreatedApp {
  app: AppEntity;
  secret: string;
}

/** The secret is shown once. */
export interface CreatedAppSecret {
  credential: AppSecretEntity;
  secret: string;
}
