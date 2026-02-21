use std::collections::hash_map::DefaultHasher;
use std::fs;
use std::hash::{Hash, Hasher};
use std::io;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct PublishedPlugin {
    pub name: String,
    pub version: String,
    pub manifest: String,
    pub signature: String,
}

#[derive(Debug)]
pub struct PluginRegistryService {
    root: PathBuf,
    secret: String,
}

impl PluginRegistryService {
    pub fn new(root: impl Into<PathBuf>, secret: impl Into<String>) -> Self {
        Self {
            root: root.into(),
            secret: secret.into(),
        }
    }

    pub fn publish(
        &self,
        name: &str,
        version: &str,
        manifest: &str,
    ) -> io::Result<PublishedPlugin> {
        fs::create_dir_all(&self.root)?;
        let signature = self.sign(manifest);
        let base = self.root.join(format!("{name}-{version}"));
        fs::write(base.with_extension("json"), manifest)?;
        fs::write(base.with_extension("sig"), &signature)?;

        Ok(PublishedPlugin {
            name: name.to_string(),
            version: version.to_string(),
            manifest: manifest.to_string(),
            signature,
        })
    }

    pub fn verify(&self, manifest: &str, signature: &str) -> bool {
        self.sign(manifest) == signature
    }

    pub fn load_and_verify(&self, name: &str, version: &str) -> io::Result<bool> {
        let base = self.root.join(format!("{name}-{version}"));
        let manifest = fs::read_to_string(base.with_extension("json"))?;
        let signature = fs::read_to_string(base.with_extension("sig"))?;
        Ok(self.verify(&manifest, &signature))
    }

    fn sign(&self, manifest: &str) -> String {
        let mut hasher = DefaultHasher::new();
        self.secret.hash(&mut hasher);
        manifest.hash(&mut hasher);
        format!("{:x}", hasher.finish())
    }
}

pub fn publish_from_file(
    service: &PluginRegistryService,
    manifest_path: impl AsRef<Path>,
    name: &str,
    version: &str,
) -> io::Result<PublishedPlugin> {
    let manifest = fs::read_to_string(manifest_path)?;
    service.publish(name, version, &manifest)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn publish_and_verify_roundtrip() {
        let dir = "registry-test";
        let service = PluginRegistryService::new(dir, "secret-key");
        let published = service
            .publish("notes", "0.1.0", "{\"name\":\"notes\"}")
            .expect("publish should work");

        assert!(service.verify(&published.manifest, &published.signature));
        assert!(service
            .load_and_verify(&published.name, &published.version)
            .expect("stored plugin should verify"));

        let _ = fs::remove_dir_all(dir);
    }
}
