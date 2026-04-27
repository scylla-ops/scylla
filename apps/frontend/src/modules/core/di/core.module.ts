import { CoreGrpcTransport } from '@core/infrastructure/grpc/core-grpc-transport.ts';

const grpcTransport: CoreGrpcTransport = new CoreGrpcTransport();

export const CoreModule = {
  data: { grpcTransport: grpcTransport },
};
