use crate::error::CliResult;
use crate::module::to_toml;
use cargo_edit::{Dependency, Manifest};

pub fn insert_into_table(
    manifest: &mut Manifest,
    table_path: &[String],
    dep: &Dependency,
) -> CliResult<()> {
    let table = manifest.get_table(table_path).unwrap();

    if table[&dep.name].is_none() {
        // insert a new entry
        let (name, new_dependency) = to_toml(&dep);
        // let root = manifest.data.as_table_mut();
        let deps = table.as_table_mut().unwrap();
        let _ = deps
            .entry(&name)
            .or_insert(toml_edit::Item::Table(new_dependency));
    } /*else {
          // update an existing entry
          merge_dependencies(&mut table[&dep.name], dep);
          if let Some(t) = table.as_inline_table_mut() {
              t.fmt()
          }
      }*/
    Ok(())
}
