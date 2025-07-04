use crate::api::v1::modules::teams::service::TeamService;
use axum::extract::State;
use axum::response::IntoResponse;
use std::sync::Arc;

type TeamState = State<Arc<TeamService>>;

pub struct TeamController {}

impl TeamController {
    pub async fn create_team(
        State(service): TeamState,
        //ValidatedJson(req): ValidatedJson<NewUserRequest>,
    ) -> impl IntoResponse {
        unimplemented!();
        "!!!!"
    }
}
