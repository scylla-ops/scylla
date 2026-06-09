import { LoginModule } from '@/modules/features/login/di/login.module.ts';
import { MarketplaceModule } from '@/modules/features/marketplace/di/marketplace.module.ts';
import { PipelineModule } from '@/modules/features/pipeline/di/pipeline.module.ts';
import { OrganizationModule } from '@/modules/features/organization/di/organization.module.ts';
import { ProjectModule } from '@/modules/features/project/di/project.module.ts';
import { JobsModule } from '@/modules/features/jobs/di/jobs.module.ts';
import { UserModule } from '@/modules/features/user/di/user.module.ts';
import { AppsModule } from '@/modules/features/apps/di/apps.module.ts';
import { AgentsModule } from '@/modules/features/agents/di/agents.module.ts';
import { SecretModule } from '@/modules/features/secret/di/secret.module.ts';
import { PermissonModule } from '@/modules/features/permission/di/permisson.module.ts';

export const dependencies = {
  login: LoginModule.domain,
  marketplace: MarketplaceModule.domain,
  pipeline: PipelineModule.domain,
  organization: OrganizationModule.domain,
  project: ProjectModule.domain,
  user: UserModule.domain,
  jobs: JobsModule.domain,
  apps: AppsModule.domain,
  agents: AgentsModule.domain,
  secret: SecretModule.domain,
  authz: PermissonModule.domain,
};

export type Dependencies = typeof dependencies;
