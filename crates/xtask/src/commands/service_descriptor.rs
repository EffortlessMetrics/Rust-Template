use anyhow::Result;
use serde_yaml::Value;
use std::fs;
use std::path::Path;

/// Generate a Backstage-like `catalog-info.yaml` from `specs/service_metadata.yaml`.
pub fn run(format: &str) -> Result<()> {
    let spec_text = fs::read_to_string("specs/service_metadata.yaml")
        .expect("specs/service_metadata.yaml must exist");

    let spec: Value = serde_yaml::from_str(&spec_text)?;

    // Build a minimal Backstage Component structure
    let mut component = serde_yaml::Mapping::new();

    component.insert(Value::from("apiVersion"), Value::from("backstage.io/v1alpha1"));
    component.insert(Value::from("kind"), Value::from("Component"));

    // metadata
    let mut metadata = serde_yaml::Mapping::new();
    if let Some(Value::String(sid)) = spec.get("service_id") {
        metadata.insert(Value::from("name"), Value::from(sid.clone()));
    }

    if let Some(Value::String(desc)) = spec.get("description") {
        metadata.insert(Value::from("description"), Value::from(desc.clone()));
    }

    if let Some(Value::Sequence(tags)) = spec.get("tags") {
        metadata.insert(Value::from("tags"), Value::from(tags.clone()));
    }

    // annotations
    let mut annotations = serde_yaml::Mapping::new();
    if let Some(Value::Mapping(links)) = spec.get("links")
        && let Some(Value::String(repo)) = links.get(Value::from("repo"))
    {
        annotations.insert(Value::from("github.com/project-slug"), Value::from(repo.clone()));
    }

    if !annotations.is_empty() {
        metadata.insert(Value::from("annotations"), Value::from(annotations));
    }

    component.insert(Value::from("metadata"), Value::from(metadata));

    // spec section
    let mut spec_map = serde_yaml::Mapping::new();
    spec_map.insert(Value::from("type"), Value::from("service"));
    spec_map.insert(Value::from("lifecycle"), Value::from("production"));
    if let Some(Value::Mapping(ownership)) = spec.get("ownership")
        && let Some(Value::String(team)) = ownership.get(Value::from("team"))
    {
        spec_map.insert(Value::from("owner"), Value::from(format!("team:{}", team)));
    }
    spec_map.insert(Value::from("system"), Value::from("platform"));

    component.insert(Value::from("spec"), Value::from(spec_map));

    // Serialize to YAML
    let out = Value::from(component);
    let yaml = serde_yaml::to_string(&out)?;

    // Ensure output dir
    let out_dir = Path::new("generated");
    fs::create_dir_all(out_dir)?;
    let out_path = out_dir.join("catalog-info.yaml");
    fs::write(&out_path, yaml)?;

    println!("Wrote {}", (out_path.display()));
    println!("Format requested: {}", format);

    Ok(())
}
