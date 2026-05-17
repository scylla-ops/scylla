use scylla_core::domain::value_objects::{PaginationMetadata, PaginationParams};
use scylla_protocol::services::common::{
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn proto_to_domain_with_valid_values() {
        let proto = ProtoPaginationRequest {
            page: 2,
            page_size: 25,
        };
        let result = proto_to_domain_pagination(Some(proto)).unwrap();
        assert_eq!(result.page(), 2);
        assert_eq!(result.page_size(), 25);
    }

    #[test]
    fn proto_to_domain_zero_page_defaults_to_1() {
        let proto = ProtoPaginationRequest {
            page: 0,
            page_size: 10,
        };
        let result = proto_to_domain_pagination(Some(proto)).unwrap();
        assert_eq!(result.page(), 1);
    }

    #[test]
    fn proto_to_domain_zero_page_size_uses_default() {
        let proto = ProtoPaginationRequest {
            page: 1,
            page_size: 0,
        };
        let result = proto_to_domain_pagination(Some(proto)).unwrap();
        assert_eq!(result.page_size(), PaginationParams::DEFAULT_PAGE_SIZE);
    }

    #[test]
    fn proto_to_domain_none_returns_none() {
        assert!(proto_to_domain_pagination(None).is_none());
    }

    #[test]
    fn domain_to_proto_metadata_maps_fields() {
        let params = PaginationParams::new(2, 20).unwrap();
        let metadata = PaginationMetadata::new(&params, 100);
        let proto = domain_to_proto_metadata(&metadata);

        assert_eq!(proto.total_count, 100);
        assert_eq!(proto.page, 2);
        assert_eq!(proto.page_size, 20);
        assert_eq!(proto.total_pages, 5);
        assert!(proto.has_next);
        assert!(proto.has_previous);
    }
}
