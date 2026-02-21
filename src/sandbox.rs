use std::collections::BTreeSet;
use std::fs;

#[derive(Debug, Clone)]
pub struct PluginManifest {
    pub name: String,
    pub wasm_path: String,
    pub permissions: BTreeSet<String>,
}

pub struct WasmSandbox;

impl WasmSandbox {
    pub fn load_manifest(path: &str) -> Result<PluginManifest, String> {
        // Simple parser to avoid external deps.
        let src = fs::read_to_string(path).map_err(|e| e.to_string())?;
        let name = extract_json_string(&src, "name").ok_or("name missing")?;
        let wasm_path = extract_json_string(&src, "wasm_path").ok_or("wasm_path missing")?;
        let permissions = extract_json_array(&src, "permissions")
            .into_iter()
            .collect::<BTreeSet<_>>();
        Ok(PluginManifest {
            name,
            wasm_path,
            permissions,
        })
    }

    pub fn execute(&self, manifest: &PluginManifest, capability: &str) -> Result<i32, String> {
        if !manifest.permissions.contains(capability) {
            return Err(format!(
                "permission denied for {} in plugin {}",
                capability, manifest.name
            ));
        }
        // Design choice: placeholder execution contract for demo; hook Wasmtime in production build.
        if !std::path::Path::new(&manifest.wasm_path).exists() {
            return Err(format!("plugin wasm file missing for {}", manifest.name));
        }
        Ok(0)
    }
}

fn extract_json_string(src: &str, key: &str) -> Option<String> {
    let token = format!("\"{}\"", key);
    let (_, tail) = src.split_once(&token)?;
    let (_, tail) = tail.split_once(':')?;
    let start = tail.find('"')? + 1;
    let rest = &tail[start..];
    let end = rest.find('"')?;
    Some(rest[..end].to_string())
}

fn extract_json_array(src: &str, key: &str) -> Vec<String> {
    let token = format!("\"{}\"", key);
    let Some((_, tail)) = src.split_once(&token) else {
        return Vec::new();
    };
    let Some((_, tail)) = tail.split_once('[') else {
        return Vec::new();
    };
    let Some((arr, _)) = tail.split_once(']') else {
        return Vec::new();
    };
    arr.split(',')
        .map(|v| v.trim().trim_matches('"').to_string())
        .filter(|v| !v.is_empty())
        .collect()
}
