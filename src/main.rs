mod types;
mod functions;

use crate::types::*;
use crate::functions::{compare, create_index, patch};
use clap::{crate_authors, CommandFactory, FromArgMatches};

#[test]
use clap::Parser;

#[tokio::main]
async fn main() {
    let config = log4rs::config::load_config_file("log4rs.yaml", Default::default()).unwrap();

    log4rs::init_config(config).unwrap();

    let args = parse();

    match args.cmd {
        SubCmd::Create { name, version, version_id, platform, input, index_output, assets_output } => {
            create_index(name, version, version_id, platform, input, index_output, assets_output).unwrap();
        }
        SubCmd::Compare { old_index, new_index, output, create_patch_bundle, assets_path } => {
            compare(old_index, new_index, output, create_patch_bundle, assets_path).unwrap();
        }
        SubCmd::Patch { root, patch_bundle, skip_check } => {
            patch(root, patch_bundle, skip_check).unwrap();
        }
    }
}

fn parse() -> Args {
    let mut cmd = Args::command()
        .author(crate_authors!("\n"))
        .about("增量包构建工具")
        .help_template("\
{before-help}{name} {version}
by {author-with-newline}
{about-with-newline}
{usage-heading} {usage}

{all-args}{after-help}
");

    cmd.build();

    let mut matches = cmd.get_matches();
    let res = Args::from_arg_matches_mut(&mut matches);
    res.unwrap_or_else(|e| {
        e.exit()
    })
}

#[tokio::test]
async fn test_create() {
    let config = log4rs::config::load_config_file("log4rs.yaml", Default::default()).unwrap();

    log4rs::init_config(config).unwrap();

    let args = Args::parse_from(r#"patch_builder.exe create --input .\test\artifacts_54\Builds\StandaloneWindows64 --output .\test\artifacts_54\bundle --name Fatal_Dopamine --version 0.45.20241105.498fdf7.54.166 --platform StandaloneWindows64"#.split(" "));
    
    match args.cmd {
        SubCmd::Create { name, version, version_id, platform, input, index_output, assets_output } => {
            create_index(name, version, version_id, platform, input, index_output, assets_output).unwrap();
        }
        SubCmd::Compare { old_index, new_index, output, create_patch_bundle, assets_path } => {
            compare(old_index, new_index, output, create_patch_bundle, assets_path).unwrap();
        }
        SubCmd::Patch { root, patch_bundle, skip_check } => {
            patch(root, patch_bundle, skip_check).unwrap();
        }
    }
}

#[tokio::test]
async fn test_compare() {
    let config = log4rs::config::load_config_file("log4rs.yaml", Default::default()).unwrap();

    log4rs::init_config(config).unwrap();

    let args = Args::parse_from(r#"patch_builder.exe compare --old-index .\test\artifacts_53\bundle\index.json --new-index .\test\artifacts_54\bundle\index.json --create-patch-bundle --output .\test\artifacts_54\migrate.zip --assets-path .\test\artifacts_54\bundle\assets --assets-path .\test\artifacts_53\bundle\assets"#.split(" "));
    
    match args.cmd {
        SubCmd::Create { name, version, version_id, platform, input, index_output, assets_output } => {
            create_index(name, version, version_id, platform, input, index_output, assets_output).unwrap();
        }
        SubCmd::Compare { old_index, new_index, output, create_patch_bundle, assets_path } => {
            compare(old_index, new_index, output, create_patch_bundle, assets_path).unwrap();
        }
        SubCmd::Patch { root, patch_bundle, skip_check } => {
            patch(root, patch_bundle, skip_check).unwrap();
        }
    }
}

#[tokio::test]
async fn test_patch() {
    let config = log4rs::config::load_config_file("log4rs.yaml", Default::default()).unwrap();

    log4rs::init_config(config).unwrap();

    let args = Args::parse_from(r#"patch_builder.exe patch --root .\test\artifacts_53\Patch\StandaloneWindows64 --patch-bundle .\test\artifacts_54\migrate.zip"#.split(" "));

    match args.cmd {
        SubCmd::Create { name, version, version_id, platform, input, index_output, assets_output } => {
            create_index(name, version, version_id, platform, input, index_output, assets_output).unwrap();
        }
        SubCmd::Compare { old_index, new_index, output, create_patch_bundle, assets_path } => {
            compare(old_index, new_index, output, create_patch_bundle, assets_path).unwrap();
        }
        SubCmd::Patch { root, patch_bundle, skip_check } => {
            patch(root, patch_bundle, skip_check).unwrap();
        }
    }
}