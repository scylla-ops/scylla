import { AppsRemoteDataSourceImpl } from '@/modules/features/apps/infrastructure/data/apps-remote.data-source.ts';
import { CoreModule } from '@core/di/core.module.ts';
import { DefaultAppsRepository } from '@/modules/features/apps/infrastructure/repository/default-apps.repository.ts';
import { GetAppsUseCase } from '@/modules/features/apps/domain/use-cases/get-apps.use-case.ts';
import { GetAppUseCase } from '@/modules/features/apps/domain/use-cases/get-app.use-case.ts';
import { CreateAppUseCase } from '@/modules/features/apps/domain/use-cases/create-app.use-case.ts';
import { DeleteAppUseCase } from '@/modules/features/apps/domain/use-cases/delete-app.use-case.ts';
import { SetAppActiveUseCase } from '@/modules/features/apps/domain/use-cases/set-app-active.use-case.ts';
import { ListAppSecretsUseCase } from '@/modules/features/apps/domain/use-cases/list-app-secrets.use-case.ts';
import { CreateAppSecretUseCase } from '@/modules/features/apps/domain/use-cases/create-app-secret.use-case.ts';
import { RevokeAppSecretUseCase } from '@/modules/features/apps/domain/use-cases/revoke-app-secret.use-case.ts';
import { SetAppSecretEnabledUseCase } from '@/modules/features/apps/domain/use-cases/set-app-secret-enabled.use-case.ts';

const dataSource = new AppsRemoteDataSourceImpl(CoreModule.data.grpcTransport);
const repository = new DefaultAppsRepository(dataSource);

export const AppsModule = {
  domain: {
    getApps: new GetAppsUseCase(repository),
    getApp: new GetAppUseCase(repository),
    createApp: new CreateAppUseCase(repository),
    deleteApp: new DeleteAppUseCase(repository),
    setAppActive: new SetAppActiveUseCase(repository),
    listAppSecrets: new ListAppSecretsUseCase(repository),
    createAppSecret: new CreateAppSecretUseCase(repository),
    revokeAppSecret: new RevokeAppSecretUseCase(repository),
    setAppSecretEnabled: new SetAppSecretEnabledUseCase(repository),
  },
};
