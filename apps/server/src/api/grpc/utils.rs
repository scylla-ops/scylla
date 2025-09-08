#[macro_export]
macro_rules! parse_uuid {
    ($id:expr) => {
        $id.parse().map_err(|e| {
            Status::invalid_argument(format!(
                "Failed to parse {} '{}': {}",
                stringify!($id),
                $id,
                e
            ))
        })
    };
}
