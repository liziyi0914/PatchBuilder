mod types;
mod functions;

use clap::{crate_authors, CommandFactory, FromArgMatches};

#[cfg(test)]
use clap::Parser;
use patch_builder::exec;
use patch_builder::types::*;

#[tokio::main]
async fn main() {
    let config = log4rs::config::load_config_file("log4rs.yaml", Default::default()).unwrap();

    log4rs::init_config(config).unwrap();

    let args = parse();

    exec(args.cmd, None).await.unwrap();
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

    let args = Args::parse_from(r#"patch_builder.exe create --input .\test\artifacts_54\Builds\StandaloneWindows64 --assets-output .\test\artifacts_54\bundle --index-output .\test\artifacts_54\index.json --name Fatal_Dopamine --version 0.45.20241105.498fdf7.54.166 --platform StandaloneWindows64"#.split(" "));
    
    exec(args.cmd, None).await.unwrap();
}

#[tokio::test]
async fn test_compare() {
    let config = log4rs::config::load_config_file("log4rs.yaml", Default::default()).unwrap();

    log4rs::init_config(config).unwrap();

    let args = Args::parse_from(r#"patch_builder.exe compare --old-index .\test\artifacts_53\bundle\index.json --new-index .\test\artifacts_54\bundle\index.json --create-patch-bundle --output .\test\artifacts_54\migrate.zip --assets-path .\test\artifacts_54\bundle\assets --assets-path .\test\artifacts_53\bundle\assets"#.split(" "));

    exec(args.cmd, None).await.unwrap();
}

#[tokio::test]
async fn test_patch() {
    let config = log4rs::config::load_config_file("log4rs.yaml", Default::default()).unwrap();

    log4rs::init_config(config).unwrap();

    let args = Args::parse_from(r#"patch_builder.exe patch --root .\test\artifacts_53\Patch\StandaloneWindows64 --patch-bundle .\test\artifacts_54\migrate.zip"#.split(" "));

    exec(args.cmd, None).await.unwrap();
}
