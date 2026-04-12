import { CoreGrpcTransport } from '@core/infrastructure/grpc/CoreGrpcTransport.ts';

const grpcTransport: CoreGrpcTransport = new CoreGrpcTransport();

export const CoreModule = {
  data: { grpcTransport: grpcTransport },
};
