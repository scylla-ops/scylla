macro_rules! require_permission {
    ($self:expr, $request:expr, $policy:expr) => {{
        let auth = extract_auth_context(&$request)?;
        $self
            .permission_checker
            .check(&auth.user_id, $policy)
            .await
            .map_err(domain_error_to_status)?;
        auth
    }};
}
