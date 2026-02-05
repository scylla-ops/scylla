use domain::entities::{
    OrganizationId, ProjectId, SessionId, UserId, UserOrganizationId, UserProjectId,
};
use surrealdb::RecordId;

pub trait ToRecordId {
    fn to_record_id(&self) -> RecordId;
}

macro_rules! impl_to_record_id {
    ($id_type:ident) => {
        impl ToRecordId for $id_type {
            fn to_record_id(&self) -> RecordId {
                use surrealdb::RecordIdKey;
                let record_id_key = RecordIdKey::from(self.as_str());
                RecordId::from_table_key(<$id_type>::table_name(), record_id_key)
            }
        }
    };
}

impl_to_record_id!(UserId);
impl_to_record_id!(OrganizationId);
impl_to_record_id!(ProjectId);
impl_to_record_id!(UserOrganizationId);
impl_to_record_id!(UserProjectId);
impl_to_record_id!(SessionId);
