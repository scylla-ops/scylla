/// Pull the authenticated `CallerContext` (a user or a machine App) out of the
/// tonic request.
///
/// Wrapping the pattern in a macro keeps every handler call site to a single
/// expression. The interceptor has already resolved the bearer token to the
/// right principal; permission enforcement lives inside each use case, so
/// handlers only forward the caller.
macro_rules! caller {
    ($request:expr) => {{ extract_auth_context(&$request)?.caller }};
}
