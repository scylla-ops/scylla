import type { DomainRegistry } from '@platform/di';
import type { ScyllaModule } from '@platform/routing';
import { LoginModule } from '@/modules/features/login/login.module.ts';
import { MarketplaceModule } from '@/modules/features/marketplace/marketplace.module.ts';
import { PipelineModule } from '@/modules/features/pipeline/pipeline.module.ts';
import { OrganizationModule } from '@/modules/features/organization/organization.module.ts';
import { ProjectModule } from '@/modules/features/project/project.module.ts';
import { JobsModule } from '@/modules/features/jobs/jobs.module.ts';
import { UserModule } from '@/modules/features/user/user.module.ts';
import { AppsModule } from '@/modules/features/apps/apps.module.ts';
import { AgentsModule } from '@/modules/features/agents/agents.module.ts';
import { SecretModule } from '@/modules/features/secret/secret.module.ts';
import { RolesModule } from '@/modules/features/roles/roles.module.ts';
import { TriggersModule } from '@/modules/features/triggers/triggers.module.ts';
import { DashboardModule } from '@/modules/features/dashboard/dashboard.module.ts';
import { MembershipModule } from '@/modules/features/membership/membership.module.ts';

/**
 * The composition root: the one place that knows every module.
 *
 * Registration order decides sidebar and route order within a section, so the
 * list reads roughly top-to-bottom as the app does.
 *
 * These are the modules' `<name>.module.ts` declarations, never their `index.ts`
 * public API — the barrels re-export UI, and importing one here would pull every
 * page into the initial chunk and undo the lazy routes.
 */
export const modules = [
  LoginModule,
  DashboardModule,
  ProjectModule,
  PipelineModule,
  JobsModule,
  TriggersModule,
  SecretModule,
  MembershipModule,
  OrganizationModule,
  UserModule,
  RolesModule,
  AgentsModule,
  AppsModule,
  MarketplaceModule,
] as const satisfies readonly ScyllaModule[];

/** Module id -> its use cases, for `useModuleDomain`. */
export const dependencies: DomainRegistry = Object.fromEntries(
  modules.map(module => [module.id, module.domain]),
);
