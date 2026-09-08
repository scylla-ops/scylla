import { ScyllaGrpcTransport } from './scylla-grpc-transport.ts';

export { ScyllaGrpcTransport } from './scylla-grpc-transport.ts';

/**
 * The single gRPC-Web transport every feature's data sources are wired to.
 *
 * It lives in `platform/` rather than in the composition root so that a feature
 * can declare its own dependencies without importing the app that assembles
 * them — which is what previously made every feature and `core/` mutually
 * dependent. Data sources take the transport as a *type* only, so importing
 * this module for the type alone costs nothing at runtime.
 */
export const grpcTransport = new ScyllaGrpcTransport();
