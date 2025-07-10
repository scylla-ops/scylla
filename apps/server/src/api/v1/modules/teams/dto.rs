use crate::api::v1::models::teams::Team;
use diesel::{AsChangeset, Insertable};
use protocol::{Deserialize, Serialize};
use validator::Validate;

#[derive(Deserialize, Validate)]
pub struct NewTeamRequest {
    #[validate(length(
        min = 1,
        max = 255,
        message = "Team name must be between 1 and 255 characters"
    ))]
    pub name: String,
}

// DB only
#[derive(Insertable, Deserialize, Validate)]
#[diesel(table_name = crate::database::schema::teams)]
pub struct NewTeam {
    pub name: String,
}

impl TryFrom<NewTeamRequest> for NewTeam {
    type Error = anyhow::Error;

    fn try_from(req: NewTeamRequest) -> anyhow::Result<Self> {
        req.validate()?;
        Ok(Self { name: req.name })
    }
}

#[derive(Serialize)]
pub struct TeamResponse {
    pub uuid: uuid::Uuid,
    pub name: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

impl From<Team> for TeamResponse {
    fn from(value: Team) -> Self {
        Self {
            uuid: value.id,
            name: value.name,
            created_at: value.created_at,
            updated_at: value.updated_at,
        }
    }
}

#[derive(Deserialize, Validate)]
pub struct UpdateTeamRequest {
    #[validate(length(
        min = 1,
        max = 255,
        message = "Team name must be between 1 and 255 characters"
    ))]
    pub name: Option<String>,
}

#[derive(AsChangeset, Deserialize)]
#[diesel(table_name = crate::database::schema::teams)]
pub struct UpdateTeam {
    pub name: Option<String>,
    pub updated_at: chrono::NaiveDateTime,
}

impl TryFrom<UpdateTeamRequest> for UpdateTeam {
    type Error = anyhow::Error;

    fn try_from(req: UpdateTeamRequest) -> anyhow::Result<Self> {
        req.validate()?;
        Ok(Self {
            name: req.name,
            updated_at: chrono::Utc::now().naive_utc(),
        })
    }
}
