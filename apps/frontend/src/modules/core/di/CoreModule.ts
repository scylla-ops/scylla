import { GrpcTransport } from '@core/infrastructure/grpc/GrpcTransport.ts';

const grpcTransport: GrpcTransport = new GrpcTransport();

export const CoreModule = {
  data: { grpcTransport: grpcTransport },
};
