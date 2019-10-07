use crate::error::{CliError, CliResult};
use crate::metadata::SubstrateMetadata;

use std::fs;
use std::path::Path;

use inflector;
use regex::Regex;

pub fn add_module_to_runtime(
    manifest_path: &Path,
    mod_metadata: SubstrateMetadata,
) -> CliResult<()> {
    let runtime_lib_path = manifest_path.parent().unwrap().join("src").join("lib.rs");
    let mod_name = mod_metadata.module_name();

    let module_trait_existing = Regex::new(
        format!(
            r"(?x)
                [^//]impl\s+{}::Trait\s+for\s+Runtime\s+\{{
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

    let mut module_trait_impl = format!("impl {}::Trait for Runtime {{ ", mod_name);
    //TODO: loop & extract key-value pairs
    module_trait_impl.push_str("type Currenty = Balances; ");
    module_trait_impl.push_str("type Event = Event; ");
    module_trait_impl.push_str("}");

    let module_config = format!(
        r"
        {}: {}::{{Module, Call, Storage, Event<T>}},",
        inflector::cases::titlecase::to_title_case(&mod_name),
        mod_name
    );

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
