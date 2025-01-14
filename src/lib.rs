use crate::functions::{compare, create_index, patch};
use crate::types::{Status, StatusReport, SubCmd};
use anyhow::{bail, Result};
use tokio::sync::mpsc::Sender;

pub mod types;
pub mod functions;

pub async fn exec(cmd: SubCmd, tx_opt: Option<Sender<StatusReport>>) -> Result<()> {
    let res = match cmd {
        SubCmd::Create { name, version, version_id, platform, input, index_output, assets_output } => {
            create_index(name, version, version_id, platform, input, index_output, assets_output, tx_opt.clone()).await
        }
        SubCmd::Compare { old_index, new_index, output, create_patch_bundle, assets_path } => {
            compare(old_index, new_index, output, create_patch_bundle, assets_path, tx_opt.clone()).await
        }
        SubCmd::Patch { root, patch_bundle, skip_check } => {
            patch(root, patch_bundle, skip_check, tx_opt.clone()).await
        }
    };

    if let Err(e) = res {
        if let Some(tx) = tx_opt {
            tx.send(StatusReport {
                status: Status::Failure,
                sub_tasks: vec![]
            }).await?;
        }
        bail!(e);
    }

    Ok(())
}
