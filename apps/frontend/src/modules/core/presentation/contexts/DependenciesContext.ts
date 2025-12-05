import { createContext } from 'react';
import { LoginModule } from '@core/di/login/LoginModule.ts';
import { MarketplaceModule } from '@core/di/marketplace/MarketplaceModule.ts';
import { CoreModule } from '@core/di/core/CoreModule.ts';
import { PipelineDashboardModule } from '@core/di/pipeline-dashboard/PipelineDashboardModule.ts';

//TODO: better DI (use lib like inversifyJS or make our own one with decorator and auto injection)
class Dependencies {
  public readonly login = LoginModule.domain;
  public readonly marketplace = MarketplaceModule.domain;
  public readonly core = CoreModule.domain;
  public readonly pipelineDashboard = PipelineDashboardModule.domain;
}

export const DependenciesContext = createContext<Dependencies | null>(new Dependencies());
