//! The adapter's two moves, the same for every handler: the caller comes from the interceptor,
//! the request becomes its command or query through `Parse`, and a `DomainError` becomes a
//! `Status`. A handler body is one of these calls and the response.

use crate::grpc::convert::Parse;
use crate::grpc::mappers::domain_error_to_status;
use crate::grpc::middleware::extract_auth_context;
use scylla_extension::{Actions, Command, Fetch, Persist, Prepare, Query, Run};
use tonic::{Request, Status};

pub async fn send<Req, R>(
    actions: &Actions,
    runner: &R,
    request: Request<Req>,
) -> Result<<Req::Into as Command>::Committed, Status>
where
    Req: Parse,
    Req::Into: Command,
    R: Run<Prepare<Req::Into>> + Run<Persist<Req::Into>>,
{
    let caller = extract_auth_context(&request)?.caller;
    let command = request.into_inner().parse()?;
    actions
        .send(runner, &caller, command)
        .await
        .map_err(domain_error_to_status)
}

pub async fn query<Req, R>(
    actions: &Actions,
    runner: &R,
    request: Request<Req>,
) -> Result<<Req::Into as Query>::Output, Status>
where
    Req: Parse,
    Req::Into: Query,
    R: Run<Fetch<Req::Into>>,
{
    let caller = extract_auth_context(&request)?.caller;
    let query = request.into_inner().parse()?;
    actions
        .query(runner, &caller, query)
        .await
        .map_err(domain_error_to_status)
}
