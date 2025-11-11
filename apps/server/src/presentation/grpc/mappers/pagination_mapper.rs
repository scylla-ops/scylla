use crate::domain::value_objects::{PaginationMetadata, PaginationParams};
use protocol::services::common::{
    PaginationMetadata as ProtoPaginationMetadata, PaginationRequest as ProtoPaginationRequest,
};

/// Convert proto PaginationRequest to domain PaginationParams
pub fn proto_to_domain_pagination(
    proto: Option<ProtoPaginationRequest>,
) -> Option<PaginationParams> {
    proto.and_then(|p| PaginationParams::new(p.page, p.page_size).ok())
}

/// Convert domain PaginationMetadata to proto PaginationMetadata
pub fn domain_to_proto_metadata(metadata: PaginationMetadata) -> ProtoPaginationMetadata {
    ProtoPaginationMetadata {
        // Safely clamp to u32::MAX to prevent overflow, because json serialization can't handle u64
        total_count: std::cmp::min(metadata.total_count(), u32::MAX as u64) as u32,
        page: metadata.page(),
        page_size: metadata.page_size(),
        total_pages: metadata.total_pages(),
        has_next: metadata.has_next(),
        has_previous: metadata.has_previous(),
    }
}
