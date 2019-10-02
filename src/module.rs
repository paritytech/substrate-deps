use cargo_edit::Dependency;

/// Convert dependency to TOML
///
/// Returns a tuple with the dependency's name and either the version as a `String`
/// or the path/git repository as an `InlineTable`.
/// (If the dependency is set as `optional` or `default-features` is set to `false`,
/// an `InlineTable` is returned in any case.)
pub fn to_toml(dep: &Dependency) -> (String, toml_edit::Table) {
    let mut data = toml_edit::Table::new();
    data["package"] = toml_edit::value(dep.name.clone());
    data["version"] = toml_edit::value(dep.version().unwrap());
    data["default-features"] = toml_edit::value(false);
    data["registry"] = toml_edit::value("substrate-mods");

    ("dependencies.template".to_string(), data)

    // let data: toml_edit::Item = match (dep.optional, dep.default_features, dep.source.clone()) {
    //     // Extra short when version flag only
    //     (
    //         false,
    //         true,
    //         DependencySource::Version {
    //             version: Some(v),
    //             path: None,
    //             registry: None,
    //         },
    //     ) => toml_edit::value(v),
    //     // Other cases are represented as an inline table
    //     (optional, default_features, source) => {
    //         let mut data = toml_edit::InlineTable::default();

    //         match source {
    //             DependencySource::Version {
    //                 version,
    //                 path,
    //                 registry,
    //             } => {
    //                 if let Some(v) = version {
    //                     data.get_or_insert("version", v);
    //                 }
    //                 if let Some(p) = path {
    //                     data.get_or_insert("path", p);
    //                 }
    //                 if let Some(r) = registry {
    //                     data.get_or_insert("registry", r);
    //                 }
    //             }
    //             DependencySource::Git(v) => {
    //                 data.get_or_insert("git", v);
    //             }
    //         }
    //         if dep.optional {
    //             data.get_or_insert("optional", optional);
    //         }
    //         if !dep.default_features {
    //             data.get_or_insert("default-features", default_features);
    //         }

    //         data.fmt();
    //         toml_edit::value(toml_edit::Value::InlineTable(data))
    //     }
    // };

    // (dep.name.clone(), data)
}
