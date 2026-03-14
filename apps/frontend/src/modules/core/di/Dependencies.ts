import { LoginModule } from '@core/di/login/LoginModule.ts';
import { MarketplaceModule } from '@core/di/marketplace/MarketplaceModule.ts';
import { PipelineDashboardModule } from '@core/di/pipeline-dashboard/PipelineDashboardModule.ts';
import { PipelineCreationModule } from '@core/di/pipeline-creation/PipelineCreationModule.ts';
import { OrganizationModule } from '@core/di/organization/OrganizationModule.ts';

export const dependencies = {
  login: LoginModule.domain,
  marketplace: MarketplaceModule.domain,
  pipelineDashboard: PipelineDashboardModule.domain,
  pipelineCreation: PipelineCreationModule.domain,
  organization: OrganizationModule.domain,
};

export type Dependencies = typeof dependencies;
