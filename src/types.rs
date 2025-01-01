use clap::{Parser, Subcommand};
use serde::{Deserialize, Serialize};

#[derive(Parser)]
#[command(version, author, about, long_about = None)]
pub struct Args {
    #[command(subcommand)]
    pub cmd: SubCmd,
}

#[derive(Subcommand)]
pub enum SubCmd {
    /// 分析全量包
    Create {
        #[arg(long, help = "项目名称")]
        name: Option<String>,
        #[arg(long, help = "版本号")]
        version: Option<String>,
        #[arg(long, help = "版本编号")]
        version_id: Option<u64>,
        #[arg(long, help = "平台标识")]
        platform: Option<String>,
        #[arg(long, required = true, help = "根目录")]
        input: String,
        #[arg(long, help = "索引路径")]
        index_output: String,
        #[arg(long, help = "资源目录")]
        assets_output: String,
    },
    /// 构建增量包
    Compare {
        #[arg(long, required = true, help = "旧版Index文件")]
        old_index: String,
        #[arg(long, required = true, help = "新版版Index文件")]
        new_index: String,
        #[arg(long, help = "迁移包路径")]
        output: Option<String>,
        #[arg(long, help = "是否创建Patch文件", default_value = "false")]
        create_patch_bundle: bool,
        #[arg(long, help = "资源根目录")]
        assets_path: Vec<String>,
    },
    /// 执行增量更新
    Patch {
        #[arg(long, required = true, help = "根目录")]
        root: String,
        #[arg(long, required = true, help = "Patch文件")]
        patch_bundle: String,
        #[arg(long, help = "跳过检查", default_value = "false")]
        skip_check: bool,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileItem {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_dir: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hash: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub size: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Index {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version_id: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub platform: Option<String>,
    pub files: Vec<FileItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Patch {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version_id: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub platform: Option<String>,
    pub migrations: Vec<Migrate>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Migrate {
    Add(FileItem),
    Delete(FileItem),
}