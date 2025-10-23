use crate::config::casbin::CASBIN_MODEL;
use casbin::{CoreApi, DefaultModel, Enforcer, MgmtApi, RbacApi};
use std::sync::{Arc, OnceLock};
use tokio::sync::RwLock;

/// global enforcer instance wrapped in Arc<RwLock<>> for thread-safe access
static ENFORCER: OnceLock<Arc<RwLock<Enforcer>>> = OnceLock::new();

/// initialize the global casbin enforcer
pub async fn init_enforcer() -> anyhow::Result<()> {
    let model = DefaultModel::from_str(CASBIN_MODEL).await?;
    let adapter = crate::api::grpc::rbac::adapter::new_adapter();
    let enforcer = Enforcer::new(model, adapter).await?;

    ENFORCER
        .set(Arc::new(RwLock::new(enforcer)))
        .map_err(|_| anyhow::anyhow!("Enforcer already initialized"))?;

    tracing::info!("Casbin enforcer initialized");
    Ok(())
}

/// get a reference to the global enforcer
pub fn get_enforcer() -> Arc<RwLock<Enforcer>> {
    ENFORCER
        .get()
        .expect("Enforcer not initialized. Call init_enforcer first.")
        .clone()
}

/// check if a user has permission to perform an action on a resource in a domain
pub async fn enforce(
    user_id: &str,
    domain_id: &str,
    resource: &str,
    action: &str,
) -> anyhow::Result<bool> {
    let enforcer = get_enforcer();
    let e = enforcer.read().await;
    
    let result = e
        .enforce((user_id, domain_id, resource, action))
        .map_err(|e| anyhow::anyhow!("Enforcement error: {}", e))?;
    
    Ok(result)
}

/// add a role assignment for a user in a domain
#[allow(dead_code)]
pub async fn add_role_for_user(
    user_id: &str,
    role: &str,
    domain_id: &str,
) -> anyhow::Result<bool> {
    let enforcer = get_enforcer();
    let mut e = enforcer.write().await;
    
    // add grouping policy: g, user, role, domain
    let result = e
        .add_grouping_policy(vec![user_id.to_string(), role.to_string(), domain_id.to_string()])
        .await
        .map_err(|e| anyhow::anyhow!("Failed to add role: {}", e))?;
    
    Ok(result)
}

/// remove a role assignment for a user in a domain
#[allow(dead_code)]
pub async fn remove_role_for_user(
    user_id: &str,
    role: &str,
    domain_id: &str,
) -> anyhow::Result<bool> {
    let enforcer = get_enforcer();
    let mut e = enforcer.write().await;
    
    // remove grouping policy: g, user, role, domain
    let result = e
        .remove_grouping_policy(vec![user_id.to_string(), role.to_string(), domain_id.to_string()])
        .await
        .map_err(|e| anyhow::anyhow!("Failed to remove role: {}", e))?;
    
    Ok(result)
}

/// add policies for a user in a domain with specific resource and actions
pub async fn add_policies_for_user(
    user_id: &str,
    domain_id: &str,
    resource: &str,
    actions: Vec<&str>,
) -> anyhow::Result<bool> {
    let enforcer = get_enforcer();
    let mut e = enforcer.write().await;
    
    for action in actions {
        let policy = vec![
            user_id.to_string(),
            domain_id.to_string(),
            resource.to_string(),
            action.to_string(),
        ];
        
        e.add_policy(policy)
            .await
            .map_err(|e| anyhow::anyhow!("Failed to add policy: {}", e))?;
    }
    
    Ok(true)
}

/// remove all policies for a user in a domain for a specific resource
pub async fn remove_policies_for_user(
    user_id: &str,
    domain_id: &str,
    resource: &str,
) -> anyhow::Result<bool> {
    let enforcer = get_enforcer();
    let mut e = enforcer.write().await;
    
    // remove filtered policy where user, domain, and resource match
    let result = e
        .remove_filtered_policy(0, vec![user_id.to_string(), domain_id.to_string(), resource.to_string()])
        .await
        .map_err(|e| anyhow::anyhow!("Failed to remove policies: {}", e))?;
    
    Ok(result)
}

/// get all roles for a user in a domain
#[allow(dead_code)]
pub async fn get_roles_for_user_in_domain(
    user_id: &str,
    _domain_id: &str,
) -> anyhow::Result<Vec<String>> {
    let enforcer = get_enforcer();
    let e = enforcer.read().await;
    
    // get all roles for this user
    // note: casbin-rs may not have full domain support in get_roles_for_user
    let all_roles = e.get_roles_for_user(user_id, None);
    Ok(all_roles)
}

