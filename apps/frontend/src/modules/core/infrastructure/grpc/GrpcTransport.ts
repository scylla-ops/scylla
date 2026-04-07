import { GrpcWebFetchTransport } from '@protobuf-ts/grpcweb-transport';
import type { RpcInterceptor, RpcOptions } from '@protobuf-ts/runtime-rpc';

export class GrpcTransport {
  private readonly _transport: GrpcWebFetchTransport;

  constructor() {
    const authInterceptor: RpcInterceptor = {
      interceptUnary(next, method, input, options: RpcOptions) {
        options.meta = options.meta ?? {};
        const token = localStorage.getItem('token');
        if (token) {
          options.meta['Authorization'] = `Bearer ${token}`;
        }
        return next(method, input, options);
      },
    };
    this._transport = new GrpcWebFetchTransport({
      baseUrl: import.meta.env.VITE_API_URL ?? '',
      format: 'binary',
      interceptors: [authInterceptor],
    });
  }

  getTransport(): GrpcWebFetchTransport {
    return this._transport;
  }
}
