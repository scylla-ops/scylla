/// Per-organization resource limits enforced when the `metering` feature is on.
/// Held by the use cases that create metered resources; the values come from
/// configuration. Adding a field here is how you add a new metered limit.
#[derive(Debug, Clone, Copy)]
pub struct Quotas {
    pub max_projects_per_org: u64,
}

impl Default for Quotas {
    fn default() -> Self {
        Self {
            max_projects_per_org: 100,
        }
    }
}
