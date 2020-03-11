use crate::error::{CliError, CliResult};
use crate::manifest::pallet_alias;

use std::fs;
use std::path::Path;

use cargo_edit::Dependency;
use inflector;
use regex::Regex;

pub fn add_pallet_to_runtime(
    manifest_path: &Path,
    dependency: &Dependency,
    alias: &Option<&str>,
) -> CliResult<()> {
    let runtime_lib_path = manifest_path.parent().unwrap().join("src").join("lib.rs");
    let mod_name = &inflector::cases::camelcase::to_camel_case(pallet_alias(dependency, alias));

    let pallet_trait_existing = Regex::new(
        format!(
            r"(?xm)
                ^impl\s+{}::Trait\s+for\s+Runtime\s+\{{
                    [^\}}]+
                \}}
        ",
            mod_name
        )
        .as_ref(),
    )?;

    let construct_runtime = Regex::new(
        r"construct_runtime!\(\s+pub\s+enum\s+Runtime[^{]+\{(?P<pallets>[\s\S]+)\}\s+\);",
    )?;

    let mut pallet_trait_impl = format!("impl {}::Trait for Runtime {{ \n", mod_name);
    pallet_trait_impl.push_str(&format!(
        "	/* {} Trait config goes here */ \n",
        dependency.name
    ));
    pallet_trait_impl.push_str("}");

    let mut pallet_config = format!(
        r"
        {}: {}::{{",
        inflector::cases::pascalcase::to_pascal_case(&mod_name),
        mod_name
    );
    pallet_config.push_str(&format!(
        "	/* {} runtime config goes here */ \n",
        dependency.name
    ));
    pallet_config.push_str("},");

    let original = fs::read_to_string(&runtime_lib_path)?;
    let mut buffer = original.clone();
    buffer = if pallet_trait_existing.is_match(&original) {
        let result =
            pallet_trait_existing.replace(&original, |_caps: &regex::Captures| &pallet_trait_impl);
        result.into()
    } else {
        let mat = construct_runtime
            .find(&original)
            .ok_or_else(|| CliError::Generic("couldn't find construct_runtime call".to_owned()))?;
        buffer.insert_str(mat.start(), format!("{}\n\n", pallet_trait_impl).as_str());
        buffer
    };

    let modified = buffer.clone();
    let caps = construct_runtime
        .captures(&modified)
        .ok_or_else(|| CliError::Generic("couldn't find construct_runtime call".to_owned()))?;
    let pallets = caps.name("pallets").ok_or_else(|| {
        CliError::Generic(
            "couldn't find runtime pallets config inside construct_runtime".to_owned(),
        )
    })?;

    buffer.insert_str(pallets.end() - 2, &pallet_config);
    fs::write(runtime_lib_path, buffer)?;

    Ok(())
}
