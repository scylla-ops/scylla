use crate::domain::value_objects::ids::*;
use surrealdb::RecordId;

pub trait ToRecordId {
    fn to_record_id(&self) -> RecordId;
}

pub trait FromRecordId: Sized {
    fn from_record_id(record_id: RecordId) -> Self;
}

macro_rules! impl_record_id_conversions {
    ($id_type:ident) => {
        impl ToRecordId for $id_type {
            fn to_record_id(&self) -> RecordId {
                use surrealdb::RecordIdKey;
                let record_id_key = RecordIdKey::from(self.as_str());
                surrealdb::RecordId::from_table_key(<$id_type>::table_name(), record_id_key)
            }
        }

        impl FromRecordId for $id_type {
            fn from_record_id(record_id: RecordId) -> Self {
                Self::new(record_id.key().to_string())
            }
        }
    };
}

impl_record_id_conversions!(UserId);
impl_record_id_conversions!(OrganizationId);
impl_record_id_conversions!(ProjectId);
impl_record_id_conversions!(PipelineId);
impl_record_id_conversions!(JobId);
impl_record_id_conversions!(UserOrganizationId);
impl_record_id_conversions!(UserProjectId);
