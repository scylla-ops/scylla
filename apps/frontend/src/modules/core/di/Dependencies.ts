import { LoginModule } from '@/modules/features/login/di/LoginModule.ts';
import { MarketplaceModule } from '@/modules/features/marketplace/di/MarketplaceModule.ts';
import { PipelineDashboardModule } from '@/modules/features/pipeline-dashboard/di/PipelineDashboardModule.ts';
import { PipelineCreationModule } from '@/modules/features/pipeline-creation/di/PipelineCreationModule.ts';
import { OrganizationModule } from '@/modules/features/organization/di/OrganizationModule.ts';
import { ProjectModule } from '@/modules/features/project/di/ProjectModule.ts';
import { JobsModule } from '@/modules/features/jobs/di/JobsModule.ts';
import { UserModule } from '@/modules/features/user/di/user.module.ts';

export const dependencies = {
  login: LoginModule.domain,
  marketplace: MarketplaceModule.domain,
  pipelineDashboard: PipelineDashboardModule.domain,
  pipelineCreation: PipelineCreationModule.domain,
  organization: OrganizationModule.domain,
  project: ProjectModule.domain,
  user: UserModule.domain,
  jobs: JobsModule.domain,
};

export type Dependencies = typeof dependencies;
