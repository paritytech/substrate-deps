use crate::error::{CliError, CliResult};
use crate::manifest::module_name;
use crate::metadata::SubstrateMetadata;

use std::fs;
use std::path::Path;

use cargo_edit::Dependency;
use inflector;
use log::debug;
use regex::Regex;

pub fn add_module_to_runtime(
    manifest_path: &Path,
    mod_dependency: &Dependency,
    mod_alias: &Option<&str>,
    mod_metadata: &Option<SubstrateMetadata>,
) -> CliResult<()> {
    let runtime_lib_path = manifest_path.parent().unwrap().join("src").join("lib.rs");
    let mod_name = &inflector::cases::camelcase::to_camel_case(module_name(
        mod_dependency,
        mod_alias,
        mod_metadata,
    ));

    let module_trait_existing = Regex::new(
        format!(
            r"(?x)
                [^/]\s+impl\s+{}::Trait\s+for\s+Runtime\s+\{{
                    [^\}}]+
                \}}
        ",
            mod_name
        )
        .as_ref(),
    )?;

    let construct_runtime = Regex::new(
        r"construct_runtime!\(\s+pub\s+enum\s+Runtime[^{]+\{(?P<modules>[\s\S]+)\}\s+\);",
    )?;

    let mut module_trait_impl = format!("impl {}::Trait for Runtime {{ \n", mod_name);
    match mod_metadata
        .as_ref()
        .and_then(|meta| meta.trait_deps_defaults())
    {
        Some(trait_defaults) => {
            for trait_default in trait_defaults {
                module_trait_impl.push_str(
                    format!("\ttype {} = {};\n", trait_default.0, trait_default.1).as_ref(),
                )
            }
        }
        None => debug!("No trait defaults for module {}", mod_dependency.name),
    }
    module_trait_impl.push_str("}");

    let mut module_config = format!(
        r"
        {}: {}::{{",
        inflector::cases::pascalcase::to_pascal_case(&mod_name),
        mod_name
    );
    match mod_metadata
        .as_ref()
        .and_then(|meta| meta.module_cfg_defaults().as_ref())
    {
        Some(cfg_defaults) => {
            for cfg_default in cfg_defaults {
                module_config.push_str(format!("{}, ", cfg_default).as_ref())
            }
        }
        None => debug!("No cfg defaults for module {}", mod_dependency.name),
    }
    module_config.push_str("},");

    let mut original = fs::read_to_string(&runtime_lib_path)?;
    let mut modified = if module_trait_existing.is_match(&original) {
        let result =
            module_trait_existing.replace(&original, |_caps: &regex::Captures| &module_trait_impl);
        result.into()
    } else {
        let mat = construct_runtime
            .find(&original)
            .ok_or_else(|| CliError::Generic("couldn't find construct_runtime call".to_owned()))?;
        original.insert_str(mat.start(), format!("{}\n\n", module_trait_impl).as_str());
        original
    };

    let caps = construct_runtime
        .captures(&modified)
        .ok_or_else(|| CliError::Generic("couldn't find construct_runtime call".to_owned()))?;
    let modules = caps.name("modules").ok_or_else(|| {
        CliError::Generic(
            "couldn't find runtime modules config inside construct_runtime".to_owned(),
        )
    })?;

    modified.insert_str(modules.end() - 2, &module_config);

    fs::write(runtime_lib_path, modified)?;

    Ok(())
}
