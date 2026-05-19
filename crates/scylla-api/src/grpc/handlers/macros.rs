/// Pull the authenticated `CallerContext::User` out of the tonic request.
///
/// Wrapping the two-line pattern in a macro keeps every handler call site to
/// a single expression and avoids repeating the `extract_auth_context →
/// CallerContext::User(...)` shape. Permission enforcement now lives inside
/// each use case, so handlers only need to forward the caller.
macro_rules! caller {
    ($request:expr) => {{
        let auth = extract_auth_context(&$request)?;
        scylla_core::application::CallerContext::User(auth.user_id.clone())
    }};
}
