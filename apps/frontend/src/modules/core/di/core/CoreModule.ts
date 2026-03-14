//core
import { CoreGrpcTransport } from '@core/infrastructure/grpc/CoreGrpcTransport.ts';

const coreGrpcTransport: CoreGrpcTransport = new CoreGrpcTransport();

export const CoreModule = {
  data: { coreGrpcTransport: coreGrpcTransport },
};
