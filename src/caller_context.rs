//! Caller Context for MCP Handlers
//!
//! Placeholder for future authentication and authorization context.
//! Will be threaded through handler signatures to enable:
//! - JWT/OAuth2 token validation
//! - RBAC permission checks
//! - Audit logging with caller identity
//! - Rate limiting per caller

use std::collections::HashMap;

/// Placeholder struct for caller authentication/authorization context.
///
/// In future phases, this will be populated from:
/// - JWT claims (Azure AD / OAuth2)
/// - API key metadata
/// - mTLS certificate subject
///
/// # Example (future)
/// ```ignore
/// let ctx = CallerContext::from_token(&jwt)?;
/// if !ctx.has_permission("node.create") {
///     return Err(McpError::Forbidden { ... });
/// }
/// ```
#[derive(Debug, Clone, Default)]
pub struct CallerContext {
    /// Caller principal ID (e.g., service account, user ID)
    pub principal_id: Option<String>,

    /// Tenant or organization ID
    pub tenant_id: Option<String>,

    /// Granted permissions/scopes
    pub permissions: Vec<String>,

    /// Additional claims from token
    pub claims: HashMap<String, String>,

    /// Request correlation ID for distributed tracing
    pub correlation_id: Option<String>,
}

impl CallerContext {
    /// Create an anonymous/unauthenticated context.
    pub fn anonymous() -> Self {
        Self::default()
    }

    /// Create a context for internal/system calls (full permissions).
    pub fn system() -> Self {
        Self {
            principal_id: Some("system".to_string()),
            tenant_id: None,
            permissions: vec!["*".to_string()],
            claims: HashMap::new(),
            correlation_id: None,
        }
    }

    /// Check if caller has a specific permission.
    ///
    /// Returns `true` if caller has wildcard permission or the specific permission.
    pub fn has_permission(&self, permission: &str) -> bool {
        self.permissions.iter().any(|p| p == "*" || p == permission)
    }

    /// Check if caller is authenticated.
    pub fn is_authenticated(&self) -> bool {
        self.principal_id.is_some()
    }

    /// Set correlation ID for request tracing.
    pub fn with_correlation_id(mut self, id: String) -> Self {
        self.correlation_id = Some(id);
        self
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_anonymous_context() {
        let ctx = CallerContext::anonymous();
        assert!(!ctx.is_authenticated());
        assert!(!ctx.has_permission("node.create"));
    }

    #[test]
    fn test_system_context() {
        let ctx = CallerContext::system();
        assert!(ctx.is_authenticated());
        assert!(ctx.has_permission("node.create"));
        assert!(ctx.has_permission("anything"));
    }

    #[test]
    fn test_specific_permission() {
        let ctx = CallerContext {
            principal_id: Some("user-123".to_string()),
            permissions: vec!["node.query".to_string(), "node.create".to_string()],
            ..Default::default()
        };
        assert!(ctx.is_authenticated());
        assert!(ctx.has_permission("node.query"));
        assert!(ctx.has_permission("node.create"));
        assert!(!ctx.has_permission("node.delete"));
    }

    #[test]
    fn test_correlation_id() {
        let ctx = CallerContext::anonymous().with_correlation_id("req-abc-123".to_string());
        assert_eq!(ctx.correlation_id.as_deref(), Some("req-abc-123"));
    }
}
