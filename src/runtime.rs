use crate::metadata::SubstrateMetadata;
use inflector;
use regex::Regex;
use std::fs;
use std::path::Path;

pub fn patch_runtime(manifest_path: &Path, mod_metadata: SubstrateMetadata) {
    let runtime_lib_path = manifest_path.parent().unwrap().join("src").join("lib.rs");
    let mod_name = mod_metadata.module_name();

    let module_trait_existing: regex::Regex = regex::Regex::new(
        format!(
            r"(?x)
                [^//]impl\s+{}::Trait\s+for\s+Runtime\s+\{{
                    [^\}}]+
                \}}
        ",
            mod_name
        )
        .as_ref(),
    )
    .unwrap();

    let construct_runtime: regex::Regex = regex::Regex::new(
        r"construct_runtime!\(\s+pub\s+enum\s+Runtime[^{]+\{(?P<modules>[\s\S]+)\}\s+\);",
    )
    .unwrap();

    let mut module_trait_impl = format!("impl {}::Trait for Runtime {{ ", mod_name);
    module_trait_impl.push_str("type Currenty = Balances; ");
    module_trait_impl.push_str("type Event = Event; ");
    module_trait_impl.push_str("}");

    let module_config = format!(
        r"
        {}: {}::{{Module, Call, Storage, Event<T>}},",
        inflector::cases::titlecase::to_title_case(&mod_name),
        mod_name
    );

    let mut original =
        fs::read_to_string(&runtime_lib_path).expect("Unable to read runtime's lib.rs");
    let mut modified = match module_trait_existing.is_match(&original) {
        true => {
            let result = module_trait_existing
                .replace(&original, |_caps: &regex::Captures| &module_trait_impl);
            result.into()
        }
        false => {
            let mat = construct_runtime
                .find(&original)
                .expect("couldn't find construct_runtime call");
            original.insert_str(mat.start(), format!("{}\n\n", module_trait_impl).as_str());
            original
        }
    };

    let caps = construct_runtime
        .captures(&modified)
        .expect("couldn't find construct_runtime call");
    let modules = caps
        .name("modules")
        .expect("couldn't find runtime modules config inside construct_runtime");
    modified.insert_str(modules.end() - 2, format!("{}", module_config).as_str());

    fs::write(runtime_lib_path, modified).expect("Unable to write runtime's lib.rs")
}
