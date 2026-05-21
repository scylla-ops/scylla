import { AppsRemoteDataSourceImpl } from '@/modules/features/apps/infrastructure/data/apps-remote.data-source.ts';
import { CoreModule } from '@core/di/core.module.ts';
import { DefaultAppsRepository } from '@/modules/features/apps/infrastructure/repository/default-apps.repository.ts';
import { GetAppsUseCase } from '@/modules/features/apps/domain/use-cases/get-apps.use-case.ts';
import { GetAppUseCase } from '@/modules/features/apps/domain/use-cases/get-app.use-case.ts';
import { CreateAppUseCase } from '@/modules/features/apps/domain/use-cases/create-app.use-case.ts';
import { DeleteAppUseCase } from '@/modules/features/apps/domain/use-cases/delete-app.use-case.ts';

const dataSource = new AppsRemoteDataSourceImpl(CoreModule.data.grpcTransport);
const repository = new DefaultAppsRepository(dataSource);

export const AppsModule = {
  domain: {
    getApps: new GetAppsUseCase(repository),
    getApp: new GetAppUseCase(repository),
    createApp: new CreateAppUseCase(repository),
    deleteApp: new DeleteAppUseCase(repository),
  },
};
