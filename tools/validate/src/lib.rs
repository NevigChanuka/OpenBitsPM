use std::{
    collections::{BTreeMap, BTreeSet},
    fs::{self, DirEntry, File},
    io::Read,
    path::{Path, PathBuf},
};

use anyhow::{Context, Result, bail, ensure};
use serde_json::Value;
use spdx::Expression;
use url::Url;

const SCHEMA: &str = include_str!("../../../schemas/package.schema.json");
const MAX_MANIFEST_BYTES: u64 = 64 * 1024;

#[derive(Debug)]
pub struct Catalog {
    pub verified: Vec<Value>,
    pub requires_verification: Vec<String>,
}

/// Read and validate the complete manifest tree before returning any publishable records.
pub fn load_catalog(root: &Path) -> Result<Catalog> {
    ensure!(
        fs::symlink_metadata(root)?.file_type().is_dir(),
        "{} must be a real directory",
        root.display()
    );
    let schema: Value = serde_json::from_str(SCHEMA)?;
    let validator = jsonschema::options()
        .should_validate_formats(true)
        .build(&schema)
        .map_err(|error| anyhow::anyhow!("invalid package schema: {error}"))?;
    let mut paths = Vec::new();
    collect_paths(root, 0, &mut paths)?;
    ensure!(
        !paths.is_empty(),
        "{} contains no manifests",
        root.display()
    );

    let mut identities = BTreeMap::new();
    let mut verified = Vec::new();
    let mut requires_verification = Vec::new();
    for path in paths {
        let (manifest, keys) = validate_manifest(root, &path, &validator)
            .with_context(|| path.display().to_string())?;
        for key in keys {
            if let Some(previous) = identities.insert(key.clone(), path.clone()) {
                bail!("{} and {} share {key}", previous.display(), path.display());
            }
        }
        if manifest["status"] == "verified" {
            verified.push(manifest);
        } else {
            requires_verification.push(string(&manifest["id"]).to_owned());
        }
    }
    verified.sort_by(|left, right| string(&left["id"]).cmp(string(&right["id"])));
    requires_verification.sort();
    Ok(Catalog {
        verified,
        requires_verification,
    })
}

fn collect_paths(dir: &Path, depth: usize, paths: &mut Vec<PathBuf>) -> Result<()> {
    let mut entries = fs::read_dir(dir)
        .with_context(|| format!("reading {}", dir.display()))?
        .collect::<std::io::Result<Vec<DirEntry>>>()?;
    entries.sort_by_key(DirEntry::file_name);
    for entry in entries {
        let path = entry.path();
        let kind = entry.file_type()?;
        if depth < 2 && kind.is_dir() {
            collect_paths(&path, depth + 1, paths)?;
        } else if depth == 2 && kind.is_file() && path.extension().is_some_and(|ext| ext == "yaml")
        {
            paths.push(path);
        } else {
            bail!(
                "{}: unexpected manifest path; expected <letter>/<Publisher>/<Package>.yaml",
                path.display()
            );
        }
    }
    Ok(())
}

fn string(value: &Value) -> &str {
    value.as_str().expect("validated manifest string")
}

fn validate_manifest(
    root: &Path,
    path: &Path,
    validator: &jsonschema::Validator,
) -> Result<(Value, BTreeSet<String>)> {
    let mut yaml = String::new();
    File::open(path)?
        .take(MAX_MANIFEST_BYTES + 1)
        .read_to_string(&mut yaml)?;
    ensure!(
        yaml.len() as u64 <= MAX_MANIFEST_BYTES,
        "manifest exceeds 64 KiB"
    );
    let manifest: Value = serde_saphyr::from_str(&yaml).context("invalid YAML")?;
    if let Err(error) = validator.validate(&manifest) {
        bail!("schema error at ${}: {error}", error.instance_path);
    }

    let id = string(&manifest["id"]);
    let (publisher, package) = id.split_once('.').expect("validated package ID");
    let expected = root
        .join(publisher[..1].to_ascii_lowercase())
        .join(publisher)
        .join(format!("{package}.yaml"));
    ensure!(path == expected, "expected path {}", expected.display());
    for segment in [publisher, package] {
        ensure!(
            !is_reserved_name(segment),
            "reserved Windows name: {segment}"
        );
    }

    let license =
        Expression::parse(string(&manifest["license"])).context("invalid SPDX expression")?;
    ensure!(
        license.requirements().all(|requirement| {
            requirement
                .req
                .license
                .id()
                .is_some_and(|id| id.is_osi_approved())
                && requirement.req.exception.is_none()
        }),
        "every SPDX license must be OSI-approved; exceptions require a future policy"
    );

    https_url(&manifest["homepage"]).context("homepage")?;
    let repository = https_url(&manifest["repository"]).context("repository")?;
    ensure!(
        repository.query().is_none() && repository.fragment().is_none(),
        "repository must be a canonical URL"
    );
    let repository = repository.as_str().to_ascii_lowercase();
    let repository = repository.trim_end_matches('/').trim_end_matches(".git");
    let mut keys = BTreeSet::from([
        format!("package ID {}", id.to_ascii_lowercase()),
        format!("repository {repository}"),
    ]);
    if let Some(verification) = manifest["verification"].as_object() {
        for (field, value) in verification {
            https_url(value).with_context(|| format!("verification.{field}"))?;
        }
    }

    let mut methods = BTreeSet::new();
    for (platform, bindings) in manifest["platforms"]
        .as_object()
        .expect("validated platforms")
    {
        for binding in bindings.as_array().expect("validated provider bindings") {
            let provider = string(&binding["provider"]);
            let package_id = string(&binding["package_id"]);
            let architecture = string(&binding["architecture"]);
            https_url(&binding["provider_source"])
                .with_context(|| format!("{platform}.{provider}.provider_source"))?;
            ensure!(
                package_id
                    .split('/')
                    .all(|part| !matches!(part, "" | "." | ".."))
                    && (matches!(provider, "homebrew" | "github-releases")
                        || !package_id.contains('/')),
                "{platform}.{provider}: invalid provider package ID {package_id}"
            );
            let identity = format!("{provider} {}", package_id.to_ascii_lowercase());
            ensure!(
                methods.insert((platform, architecture, identity.clone())),
                "{platform}.{architecture}: duplicate provider binding for {identity}"
            );
            keys.insert(format!("provider package {identity}"));
            if let Some(installer) = binding.get("installer") {
                https_url(&installer["url"])
                    .with_context(|| format!("{platform}.{provider}.installer.url"))?;
            }
        }
    }
    Ok((manifest, keys))
}

fn https_url(value: &Value) -> Result<Url> {
    let raw = string(value);
    let authority = raw
        .strip_prefix("https://")
        .unwrap_or_default()
        .split(['/', '?', '#'])
        .next()
        .unwrap_or_default();
    ensure!(
        !authority.is_empty()
            && !authority.contains('@')
            && !raw.contains(char::is_whitespace)
            && !raw.contains('\\'),
        "expected an HTTPS URL without credentials, whitespace, or backslashes"
    );
    let url = Url::parse(raw)?;
    ensure!(
        url.scheme() == "https" && url.has_host(),
        "expected an HTTPS URL with a host"
    );
    Ok(url)
}

fn is_reserved_name(segment: &str) -> bool {
    let upper = segment.to_ascii_uppercase();
    matches!(upper.as_str(), "CON" | "PRN" | "AUX" | "NUL")
        || (matches!(upper.get(..3), Some("COM" | "LPT"))
            && matches!(upper.as_bytes().get(3..), Some([b'1'..=b'9'])))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_windows_device_names() {
        for segment in ["CoN", "prn", "Aux", "nUL", "Lpt1", "COM9"] {
            assert!(is_reserved_name(segment));
        }
        for segment in ["Com0", "LPT10", "Console", "App"] {
            assert!(!is_reserved_name(segment));
        }
    }
}
