//! The adapter's one move, the same for every handler and every RPC: the caller comes from the
//! interceptor, the request becomes its command or query through `Parse`, the engine runs it,
//! and a `DomainError` becomes a `Status`. A handler body is this call and the response.

use crate::grpc::convert::Parse;
use crate::grpc::mappers::domain_error_to_status;
use crate::grpc::middleware::extract_auth_context;
use scylla_extension::{Actions, Kind, Path};
use tonic::{Request, Status};

pub async fn run<Req, K, R>(
    actions: &Actions,
    runner: &R,
    request: Request<Req>,
) -> Result<<Req::Into as Path<K, R>>::Output, Status>
where
    Req: Parse,
    Req::Into: Path<K, R>,
    K: Kind,
{
    let caller = extract_auth_context(&request)?.caller;
    let action = request.into_inner().parse()?;
    actions
        .run(runner, &caller, action)
        .await
        .map_err(domain_error_to_status)
}
