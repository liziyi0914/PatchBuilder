use crate::functions::{compare, create_index, patch};
use crate::types::SubCmd;
use anyhow::Result;

pub mod types;
pub mod functions;

pub fn exec(cmd: SubCmd) -> Result<()> {
    match cmd {
        SubCmd::Create { name, version, version_id, platform, input, index_output, assets_output } => {
            create_index(name, version, version_id, platform, input, index_output, assets_output)?;
        }
        SubCmd::Compare { old_index, new_index, output, create_patch_bundle, assets_path } => {
            compare(old_index, new_index, output, create_patch_bundle, assets_path)?;
        }
        SubCmd::Patch { root, patch_bundle, skip_check } => {
            patch(root, patch_bundle, skip_check)?;
        }
    }

    Ok(())
}
