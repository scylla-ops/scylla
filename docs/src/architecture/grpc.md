# gRPC protocol

> 🚧 *Chapter in progress.*

All API traffic is gRPC. The browser speaks gRPC-Web; agents and internal calls speak native gRPC.

## The protocol crate

<!-- scylla-protocol: .proto files + generated Rust (prost) and TypeScript (protobuf-ts). -->

## Services

<!-- Overview of the proto services: auth, org, project, pipeline, job, app, worker stream, permission, trigger, etc. -->

## gRPC-Web for the browser

<!-- tonic-web; @protobuf-ts/grpcweb-transport; CORS. -->

## Auth interceptor

<!-- Bearer token → session or app token → CallerContext in extensions. -->
