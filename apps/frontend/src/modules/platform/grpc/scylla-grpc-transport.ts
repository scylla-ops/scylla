import { GrpcWebFetchTransport } from '@protobuf-ts/grpcweb-transport';
import { RpcError } from '@protobuf-ts/runtime-rpc';
import type { RpcInterceptor, RpcOptions } from '@protobuf-ts/runtime-rpc';

/**
 * The server percent-encodes the `grpc-message` trailer and
 * `@protobuf-ts/grpcweb-transport` does not decode it: decode each error once, here.
 */
const decodedErrors = new WeakSet<RpcError>();

function decodeRpcErrorMessage(error: unknown): void {
  if (error instanceof RpcError && !decodedErrors.has(error)) {
    decodedErrors.add(error);
    try {
      error.message = decodeURIComponent(error.message);
    } catch {
      // Not valid percent-encoding: keep the raw message.
    }
  }
}

// The same RpcError goes through response, status and trailers: decoding it once is enough.
const errorDecodeInterceptor: RpcInterceptor = {
  interceptUnary(next, method, input, options) {
    const call = next(method, input, options);
    void call.headers.catch(decodeRpcErrorMessage);
    void call.response.catch(decodeRpcErrorMessage);
    void call.status.catch(decodeRpcErrorMessage);
    void call.trailers.catch(decodeRpcErrorMessage);
    return call;
  },
  interceptServerStreaming(next, method, input, options) {
    const call = next(method, input, options);
    void call.headers.catch(decodeRpcErrorMessage);
    call.responses.onError(decodeRpcErrorMessage);
    void call.status.catch(decodeRpcErrorMessage);
    void call.trailers.catch(decodeRpcErrorMessage);
    return call;
  },
};

export class ScyllaGrpcTransport {
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
      interceptors: [authInterceptor, errorDecodeInterceptor],
    });
  }

  getTransport(): GrpcWebFetchTransport {
    return this._transport;
  }
}
