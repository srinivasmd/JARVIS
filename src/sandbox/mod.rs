use crate::policy::{Permission, Policy};

#[derive(Debug, Clone)]
pub struct ToolRequest {
    pub name: String,
    pub args: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SandboxError {
    PermissionDenied,
    ToolNotFound,
}

pub trait ToolSandbox {
    fn execute(&self, policy: &Policy, request: &ToolRequest) -> Result<String, SandboxError>;
}

#[derive(Debug, Default)]
pub struct DenyByDefaultSandbox;

impl ToolSandbox for DenyByDefaultSandbox {
    fn execute(&self, policy: &Policy, request: &ToolRequest) -> Result<String, SandboxError> {
        if !policy.allows(&Permission::ToolExec) {
            return Err(SandboxError::PermissionDenied);
        }

        match request.name.as_str() {
            "echo" => Ok(request.args.join(" ")),
            _ => Err(SandboxError::ToolNotFound),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::policy::Permission;

    #[test]
    fn sandbox_denies_without_permission() {
        let sandbox = DenyByDefaultSandbox;
        let policy = Policy::allow_list([Permission::MemoryRead]);
        let result = sandbox.execute(
            &policy,
            &ToolRequest {
                name: "echo".to_string(),
                args: vec!["hi".to_string()],
            },
        );
        assert_eq!(result, Err(SandboxError::PermissionDenied));
    }
}
