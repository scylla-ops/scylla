#[macro_export]
macro_rules! handle_diesel_result {
    // Cas avec un seul pattern - utilise if let
    ($result:expr, {
        $diesel_error:pat => $custom_error:expr $(,)?
    }) => {
        match $result {
            Ok(value) => Ok(value),
            Err(err) => {
                if let Some($diesel_error) = err.downcast_ref::<diesel::result::Error>() {
                    Err($custom_error)
                } else {
                    Err(err.into())
                }
            }
        }
    };


    // Cas avec plusieurs patterns - utilise match
    ($result:expr, {
        $($diesel_error:pat => $custom_error:expr),+ $(,)?
    }) => {
        match $result {
            Ok(value) => Ok(value),
            Err(err) => {
                match err.downcast_ref::<diesel::result::Error>() {
                    $(
                        Some($diesel_error) => Err($custom_error),
                    )+
                    _ => Err(err.into()),
                }
            }
        }
    };
}
