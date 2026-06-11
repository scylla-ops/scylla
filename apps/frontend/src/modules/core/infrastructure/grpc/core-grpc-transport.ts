import { GrpcWebFetchTransport } from '@protobuf-ts/grpcweb-transport';
import { RpcError } from '@protobuf-ts/runtime-rpc';
import type { RpcInterceptor, RpcOptions } from '@protobuf-ts/runtime-rpc';

/**
 * The gRPC spec requires servers to percent-encode the `grpc-message` trailer
 * (tonic turns spaces into %20) and leaves decoding to the client — but
 * @protobuf-ts/grpcweb-transport (2.11.1) never decodes it, so RpcError.message
 * arrives percent-encoded. Decode each error exactly once, at the transport,
 * so every consumer sees a readable message. decodeURIComponent throws on
 * input that is not valid percent-encoding (e.g. a literal '%'), in which case
 * the raw message wins.
 */
const decodedErrors = new WeakSet<RpcError>();

function decodeRpcErrorMessage(error: unknown): void {
  if (error instanceof RpcError && !decodedErrors.has(error)) {
    decodedErrors.add(error);
    try {
      error.message = decodeURIComponent(error.message);
    } catch {
      // Not valid percent-encoding — keep the raw message.
    }
  }
}

// Handlers registered here run before any attached later by callers, and the
// same RpcError instance flows through response/status/trailers, so mutating
// its message once is visible everywhere.
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

export class CoreGrpcTransport {
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
