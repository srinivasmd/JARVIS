use std::collections::HashSet;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Permission {
    Network,
    FileRead,
    FileWrite,
    MemoryRead,
    MemoryWrite,
    ToolExec,
}

#[derive(Debug, Clone)]
pub struct Policy {
    allowed: HashSet<Permission>,
}

impl Policy {
    pub fn allow_list(permissions: impl IntoIterator<Item = Permission>) -> Self {
        Self {
            allowed: permissions.into_iter().collect(),
        }
    }

    pub fn allows(&self, permission: &Permission) -> bool {
        self.allowed.contains(permission)
    }
}
