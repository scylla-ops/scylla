//! Wire to command, command outcome to wire. The handler holds none of it.

use crate::application::oauth::{AccountOutcome, GetAuthUrl, OAuthCallback, OAuthOutcome};
use crate::grpc::convert::wrap;
use scylla_proto::oauth::v1::{
    CallbackRequest, CallbackResponse, GetAuthUrlRequest, callback_response,
    callback_response::{ExistingAccount, NewAccount},
};

parse!(GetAuthUrlRequest => GetAuthUrl { state: copy });
parse!(CallbackRequest => OAuthCallback { code: copy });

impl From<OAuthOutcome> for CallbackResponse {
    fn from(outcome: OAuthOutcome) -> Self {
        let account = match outcome.account {
            AccountOutcome::New { organization_id } => {
                callback_response::Outcome::NewAccount(NewAccount {
                    organization_id: wrap(organization_id.to_string()),
                })
            }
            AccountOutcome::Existing => {
                callback_response::Outcome::ExistingAccount(ExistingAccount {})
            }
        };
        Self {
            token: outcome.token,
            user_id: wrap(outcome.user_id.to_string()),
            outcome: Some(account),
        }
    }
}
