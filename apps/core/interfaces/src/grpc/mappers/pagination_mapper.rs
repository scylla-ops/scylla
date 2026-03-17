use domain::value_objects::{PaginationMetadata, PaginationParams};
use protocol::services::common::{
    PaginationMetadata as ProtoPaginationMetadata, PaginationRequest as ProtoPaginationRequest,
};

pub fn proto_to_domain_pagination(
    proto: Option<ProtoPaginationRequest>,
) -> Option<PaginationParams> {
    proto.and_then(|p| {
        let page = if p.page == 0 { 1 } else { p.page };
        let page_size = if p.page_size == 0 {
            PaginationParams::DEFAULT_PAGE_SIZE
        } else {
            p.page_size
        };
        PaginationParams::new(page, page_size).ok()
    })
}

pub fn domain_to_proto_metadata(metadata: &PaginationMetadata) -> ProtoPaginationMetadata {
    ProtoPaginationMetadata {
        // clamp to u32::MAX to prevent overflow
        total_count: std::cmp::min(metadata.total_count(), u32::MAX as u64) as u32,
        page: metadata.page(),
        page_size: metadata.page_size(),
        total_pages: metadata.total_pages(),
        has_next: metadata.has_next(),
        has_previous: metadata.has_previous(),
    }
}
