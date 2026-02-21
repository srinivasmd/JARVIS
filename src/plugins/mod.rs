use crate::policy::Permission;

#[derive(Debug, Clone)]
pub struct PluginManifest {
    pub name: String,
    pub version: String,
    pub capabilities: Vec<String>,
    pub permissions: Vec<ManifestPermission>,
    pub source: PluginSource,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PluginSource {
    VettedRegistry,
    Local,
    Unknown,
}

#[derive(Debug, Clone)]
pub enum ManifestPermission {
    Network,
    FileRead,
    FileWrite,
    MemoryRead,
    MemoryWrite,
    ToolExec,
}

impl From<&ManifestPermission> for Permission {
    fn from(value: &ManifestPermission) -> Self {
        match value {
            ManifestPermission::Network => Permission::Network,
            ManifestPermission::FileRead => Permission::FileRead,
            ManifestPermission::FileWrite => Permission::FileWrite,
            ManifestPermission::MemoryRead => Permission::MemoryRead,
            ManifestPermission::MemoryWrite => Permission::MemoryWrite,
            ManifestPermission::ToolExec => Permission::ToolExec,
        }
    }
}

#[derive(Debug, Default)]
pub struct PluginRegistry {
    manifests: Vec<PluginManifest>,
    auto_install_enabled: bool,
}

impl PluginRegistry {
    pub fn new(auto_install_enabled: bool) -> Self {
        Self {
            manifests: Vec::new(),
            auto_install_enabled,
        }
    }

    pub fn register(&mut self, manifest: PluginManifest) -> Result<(), String> {
        if !self.auto_install_enabled && manifest.source == PluginSource::Unknown {
            return Err("plugin rejected: unknown source and auto install disabled".to_string());
        }

        if !is_manifest_vetted(&manifest) {
            return Err("plugin rejected: manifest failed vetting".to_string());
        }

        self.manifests.push(manifest);
        Ok(())
    }

    pub fn all(&self) -> &[PluginManifest] {
        &self.manifests
    }
}

pub fn is_manifest_vetted(manifest: &PluginManifest) -> bool {
    !manifest.name.is_empty()
        && !manifest.version.is_empty()
        && !manifest.capabilities.is_empty()
        && manifest.permissions.len() <= 8
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn registry_blocks_unknown_source_by_default() {
        let mut registry = PluginRegistry::new(false);
        let result = registry.register(PluginManifest {
            name: "notes".to_string(),
            version: "0.1.0".to_string(),
            capabilities: vec!["store_notes".to_string()],
            permissions: vec![ManifestPermission::MemoryRead],
            source: PluginSource::Unknown,
        });

        assert!(result.is_err());
    }

    #[test]
    fn registry_accepts_vetted_registry_source() {
        let mut registry = PluginRegistry::new(false);
        registry
            .register(PluginManifest {
                name: "notes".to_string(),
                version: "0.1.0".to_string(),
                capabilities: vec!["store_notes".to_string()],
                permissions: vec![ManifestPermission::MemoryRead],
                source: PluginSource::VettedRegistry,
            })
            .expect("vetted plugin should register");

        assert_eq!(registry.all().len(), 1);
    }
}
