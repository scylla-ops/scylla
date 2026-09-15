macro_rules! caller {
    ($request:expr) => {{ extract_auth_context(&$request)?.caller }};
}
