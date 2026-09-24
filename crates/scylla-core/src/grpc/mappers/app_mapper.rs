//! Wire to command, command outcome to wire. The handler holds none of it.

use crate::application::app::{
    CreateApp, CreateAppSecret, DeleteApp, GetApp, ListAppSecrets, ListApps, SetAppActive,
};
use crate::grpc::convert::{Parse, id, ts, valid, wrap};
use scylla_domain::domain::app::{App, AppCredential, AppName, AppSecretLabel};
use scylla_proto::app::v1::{
    App as ProtoApp, AppSecret as ProtoAppSecret, CreateAppRequest, CreateAppSecretRequest,
    DeleteAppRequest, GetAppRequest, ListAppSecretsRequest, ListAppsRequest, SetAppActiveRequest,
};
use tonic::Status;

pub fn app_to_proto(a: &App) -> ProtoApp {
    ProtoApp {
        app_id: wrap(a.id().to_string()),
        organization_id: wrap(a.organization_id().to_string()),
        name: a.name().as_str().to_string(),
        is_active: a.is_active(),
        created_at: ts(a.created_at()),
        updated_at: ts(a.updated_at()),
    }
}

pub fn app_credential_to_proto(c: &AppCredential) -> ProtoAppSecret {
    ProtoAppSecret {
        app_secret_id: wrap(c.id().to_string()),
        app_id: wrap(c.app_id().to_string()),
        label: c.label().as_str().to_string(),
        enabled: c.is_enabled(),
        created_at: ts(c.created_at()),
        updated_at: ts(c.updated_at()),
    }
}

impl Parse for CreateAppRequest {
    type Into = CreateApp;

    fn parse(self) -> Result<CreateApp, Status> {
        Ok(CreateApp {
            organization_id: id(self.organization_id, "organization_id")?,
            name: valid(self.name, AppName::new)?,
        })
    }
}

parse!(GetAppRequest => GetApp { id: id(app_id) });
parse!(ListAppsRequest => ListApps { organization_id: id(organization_id) });
parse!(DeleteAppRequest => DeleteApp { id: id(app_id) });

parse!(SetAppActiveRequest => SetAppActive {
    id: id(app_id),
    is_active: copy,
});

impl Parse for CreateAppSecretRequest {
    type Into = CreateAppSecret;

    fn parse(self) -> Result<CreateAppSecret, Status> {
        Ok(CreateAppSecret {
            app_id: id(self.app_id, "app_id")?,
            label: valid(self.label, AppSecretLabel::new)?,
        })
    }
}

parse!(ListAppSecretsRequest => ListAppSecrets { app_id: id(app_id) });

#[cfg(test)]
mod tests {
    use super::*;
    use scylla_proto::common::v1 as common;
    use tonic::Code;

    #[test]
    fn a_create_request_becomes_a_command_with_validated_fields() {
        let command = CreateAppRequest {
            organization_id: wrap("acme"),
            name: "ci-bot".into(),
        }
        .parse()
        .unwrap();

        assert_eq!(command.organization_id.as_str(), "acme");
        assert_eq!(command.name.as_str(), "ci-bot");
    }

    #[test]
    fn a_set_active_request_keeps_the_flag() {
        let command = SetAppActiveRequest {
            app_id: wrap("app-1"),
            is_active: false,
        }
        .parse()
        .unwrap();

        assert_eq!(command.id.as_str(), "app-1");
        assert!(!command.is_active);
    }

    #[test]
    fn a_missing_app_id_is_an_invalid_argument() {
        let err = GetAppRequest {
            app_id: None::<common::AppId>,
        }
        .parse()
        .unwrap_err();

        assert_eq!(err.code(), Code::InvalidArgument);
        assert_eq!(err.message(), "missing app_id");
    }

    #[test]
    fn an_invalid_name_is_an_invalid_argument() {
        let Err(err) = CreateAppRequest {
            organization_id: wrap("acme"),
            name: String::new(),
        }
        .parse() else {
            panic!("an empty name must not parse");
        };

        assert_eq!(err.code(), Code::InvalidArgument);
    }

    #[test]
    fn an_invalid_label_is_an_invalid_argument() {
        let Err(err) = CreateAppSecretRequest {
            app_id: wrap("app-1"),
            label: String::new(),
        }
        .parse() else {
            panic!("an empty label must not parse");
        };

        assert_eq!(err.code(), Code::InvalidArgument);
    }
}
