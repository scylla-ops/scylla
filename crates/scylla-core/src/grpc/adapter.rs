//! The adapter's one move, the same for every handler and every RPC: the caller comes from the
//! interceptor, the request becomes its command or query through `Parse`, the engine runs it,
//! and a `DomainError` becomes a `Status`. A handler body is this call and the response.

use crate::domain::caller::CallerContext;
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
    send(actions, runner, &caller, request.into_inner()).await
}

/// `run` for a service without the interceptor: the caller is `Anonymous`, so only a `Public`
/// action passes the authorize stage.
pub async fn run_public<Req, K, R>(
    actions: &Actions,
    runner: &R,
    request: Request<Req>,
) -> Result<<Req::Into as Path<K, R>>::Output, Status>
where
    Req: Parse,
    Req::Into: Path<K, R>,
    K: Kind,
{
    send(
        actions,
        runner,
        &CallerContext::Anonymous,
        request.into_inner(),
    )
    .await
}

async fn send<Req, K, R>(
    actions: &Actions,
    runner: &R,
    caller: &CallerContext,
    request: Req,
) -> Result<<Req::Into as Path<K, R>>::Output, Status>
where
    Req: Parse,
    Req::Into: Path<K, R>,
    K: Kind,
{
    let action = request.parse()?;
    actions
        .run(runner, caller, action)
        .await
        .map_err(domain_error_to_status)
}
