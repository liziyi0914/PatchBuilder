use std::collections::HashMap;
use std::fs;
use std::fs::{create_dir_all, File};
use std::io::{BufReader, Read, Write};
use std::path::{Path, PathBuf};
use anyhow::{bail, Result};
use log::{error, info};
use sha2::{Digest, Sha256};
use zip::write::SimpleFileOptions;
use crate::types::{FileItem, Index, Migrate, Patch};

fn get_file_size_and_hash(file_path: &Path) -> Result<(u64, String)> {
    // 打开文件
    let mut file = File::open(file_path)?;

    // 获取文件大小
    let file_size = file.metadata()?.len();

    // 创建SHA-256哈希计算器
    let mut hasher = Sha256::new();

    // 读取文件内容并计算哈希值
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)?;
    hasher.update(&buffer);

    // 获取哈希值并转换为十六进制字符串
    let hash_result = hasher.finalize();
    let hash_hex = hex::encode(hash_result);

    Ok((file_size, hash_hex))
}

/// 获取目录中所有子文件信息
fn list_files(path: PathBuf, root: PathBuf) -> Result<Vec<FileItem>> {
    let mut list = vec![];

    let mut dir = std::fs::read_dir(path)?;

    while let Some(Ok(entry)) = dir.next() {
        let p = entry.path();
        if p.is_dir() {
            let relative = p.strip_prefix(&root)?;
            list.push(FileItem {
                name: Some(relative.display().to_string()),
                is_dir: Some(true),
                hash: None,
                size: None,
            });
            let mut tmp = list_files(p.clone(), root.clone())?;
            list.append(&mut tmp);
        } else {
            let relative = p.strip_prefix(&root)?;

            list.push(FileItem {
                name: Some(relative.display().to_string()),
                is_dir: None,
                hash: None,
                size: None,
            });
        }
    }

    Ok(list)
}

fn load_file_total_info(root: PathBuf, files: &mut Vec<FileItem>) -> Result<()> {
    let count = files.iter().filter(|f| !matches!(f.is_dir, Some(true))).count();
    let mut current = 0;

    for item in files.iter_mut() {
        if matches!(item.is_dir, Some(true)) {
            continue;
        }
        current = current + 1;
        let name = item.name.clone().unwrap();
        info!("读取文件({}/{}) {}", current, count, name);
        let (size, hash) = get_file_size_and_hash(root.join(name).as_path())?;
        item.size = Some(size);
        item.hash = Some(hash);
    }

    Ok(())
}

pub fn create_index(name: Option<String>,
                version: Option<String>,
                version_id: Option<u64>,
                platform: Option<String>,
                input: String,
                index_output: String,
                assets_output: Option<String>,
) -> Result<()> {
    let input_path = PathBuf::from(input);
    let index_path = PathBuf::from(&index_output);
    let assets_path_opt = assets_output.map(|s| PathBuf::from(s));

    info!("开始载入目录 {}", input_path.display());

    let mut list = list_files(input_path.clone().into(), input_path.clone().into())?;

    load_file_total_info(input_path.clone().into(), &mut list)?;

    let index = Index {
        name,
        version,
        version_id,
        platform,
        files: list,
    };

    let parent = index_path.parent().unwrap();
    if !fs::exists(parent)? {
        fs::create_dir(parent)?;
    }

    info!("写入Index文件");
    let mut file = File::options().create(true).write(true).open(index_path)?;
    file.write(serde_json::to_string(&index)?.as_bytes())?;

    if assets_path_opt.is_none() {
        info!("Index文件创建成功");
        return Ok(());
    }

    let assets_path = assets_path_opt.unwrap();

    if !fs::exists(&assets_path)? {
        fs::create_dir(&assets_path)?;
    }

    info!("开始生成资源文件集");
    let count = index.files.iter().filter(|f| !matches!(f.is_dir, Some(true))).count();
    let mut current = 0;

    for item in index.files.iter() {
        if matches!(item.is_dir, Some(true)) {
            continue;
        }
        current = current + 1;
        let name = item.name.clone().unwrap();
        let hash = item.hash.clone().unwrap();
        let mut asset_path = assets_path.join(&hash[..2]);
        if !asset_path.exists() {
            fs::create_dir(&asset_path)?;
        }
        asset_path = asset_path.join(&hash);
        info!("复制文件({}/{}) {}  =>  {}/{}", current, count, name, &hash[..2], &hash);
        File::options().create(true).write(true).open(&asset_path)?;
        fs::copy(input_path.join(name), asset_path)?;
    }

    info!("Index文件创建成功");

    Ok(())
}

pub fn compare(old_index: String, new_index: String, output: Option<String>, create_patch_bundle: bool, assets_paths: Vec<String>,) -> Result<()> {
    let old_index_path = PathBuf::from(&old_index);
    let new_index_path = PathBuf::from(&new_index);

    let index_old: Index = serde_json::from_reader(File::open(old_index_path)?)?;
    let index_new: Index = serde_json::from_reader(File::open(new_index_path)?)?;

    info!("开始比较资源文件");

    let mut map = HashMap::new();

    for item in index_old.files.iter() {
        map.insert(item.name.clone().unwrap(), item.clone());
    }

    let mut migrations = vec![];

    let mut new_list = vec![];

    for item in index_new.files.iter() {
        if map.contains_key(item.name.as_ref().unwrap()) {
            //有同名文件/文件夹
            new_list.push(item.name.as_ref().unwrap().clone());
            let old = &map[item.name.as_ref().unwrap()];
            if old.is_dir == Some(true) {
                //原本是文件夹
                if item.is_dir == Some(true) {
                    continue;
                } else {
                    //现在是文件
                    migrations.push(Migrate::Delete(old.clone()));
                    migrations.push(Migrate::Add(item.clone()));
                    info!("变换为文件 {}: {}", item.name.as_ref().unwrap(), item.hash.as_ref().unwrap().get(0..8).unwrap());
                }
            } else {
                //原本是文件
                if item.is_dir == Some(true) {
                    //现在是文件夹
                    migrations.push(Migrate::Delete(old.clone()));
                    migrations.push(Migrate::Add(item.clone()));
                    info!("变换为文件夹 {}: {}", item.name.as_ref().unwrap(), item.hash.as_ref().unwrap().get(0..8).unwrap());
                } else {
                    if item.hash == old.hash && item.size == old.size {
                        continue;
                    } else {
                        migrations.push(Migrate::Delete(old.clone()));
                        migrations.push(Migrate::Add(item.clone()));

                        let diff_size = {
                            let n = item.size.unwrap() - old.size.unwrap();
                            if n > 0 {
                                format!(" +{}Bytes", n)
                            } else if n==0 {
                                "           ".to_string()
                            } else {
                                format!(" {}Bytes", n)
                            }
                        };

                        info!(
                            "迁移文件 {} => {}{} \t{}",
                            old.hash.as_ref().unwrap().get(0..8).unwrap(),
                            item.hash.as_ref().unwrap().get(0..8).unwrap(),
                            diff_size,
                            item.name.as_ref().unwrap(),
                        );
                    }
                }
            }
        } else {
            //无同名文件/文件夹
            migrations.push(Migrate::Add(item.clone()));
            info!("新建 {}", item.name.as_ref().unwrap());
        }
    }

    for i in new_list {
        map.remove(&i);
    }

    for (_, item) in map {
        migrations.push(Migrate::Delete(item.clone()));
        info!("删除文件 {}", item.name.as_ref().unwrap());
    }

    if !create_patch_bundle {
        info!("迁移信息: {}", serde_json::to_string_pretty(&migrations)?)
    } else {
        let patch = Patch {
            name: index_new.name.clone(),
            version: index_new.version.clone(),
            version_id: index_new.version_id.clone(),
            platform: index_new.platform.clone(),
            migrations: migrations.clone(),
        };

        if output.is_none() {
            error!("缺少参数: output");
            bail!("缺少参数: output");
        }

        info!("开始构建增量包");

        let file = File::options().write(true).create(true).open(output.unwrap())?;
        let mut zip = zip::ZipWriter::new(file);

        let options = SimpleFileOptions::default()
            .compression_method(zip::CompressionMethod::Deflated)
            .unix_permissions(0o755);

        info!("写入索引文件");

        zip.start_file("index.json", options)?;

        zip.write_all(serde_json::to_string(&patch)?.as_bytes())?;

        zip.add_directory("assets", SimpleFileOptions::default())?;

        let assets_paths = assets_paths.iter().map(|s| PathBuf::from(s)).collect::<Vec<_>>();

        for m in migrations {
            match m {
                Migrate::Add(item) => {
                    let hash = item.hash.clone().unwrap();
                    let mut p = None;
                    for pb in &assets_paths {
                        let tmp = pb.join(&hash[..2]).join(&hash);
                        if tmp.exists() {
                            p = Some(tmp);
                        }
                    }
                    if p.is_none() {
                        error!("找不到资源文件 {}: {}", item.hash.clone().unwrap(), item.name.clone().unwrap());
                        bail!("找不到资源文件 {}: {}", item.hash.clone().unwrap(), item.name.clone().unwrap());
                    }

                    info!("开始写入 {} {}Bytes", item.hash.clone().unwrap(), item.size.clone().unwrap());

                    zip.start_file(format!("assets/{}", hash), options)?;

                    let mut f = File::open(p.unwrap())?;

                    loop {
                        let mut buf = vec![0; 1024];
                        let size = f.read(&mut buf)?;
                        if size == 0 {
                            break;
                        }
                        zip.write_all(&buf[..size])?;
                    }
                }
                Migrate::Delete(_) => {}
            }
        }

        zip.finish()?;

        info!("构建成功");
    }

    Ok(())
}

pub fn patch(root: String,
         patch_bundle: String,
         skip_check: bool,
) -> Result<()> {
    let root = PathBuf::from(&root);
    let patch_file = File::open(patch_bundle)?;

    info!("开始读取增量包");

    let mut archive = zip::ZipArchive::new(patch_file).unwrap();

    let index = archive.by_name("index.json")?;

    let reader = BufReader::new(index);
    let index: Patch = serde_json::from_reader(reader)?;

    info!("应用名称: {:?}", index.name);
    info!("版本: {:?}", index.version);
    info!("版本编号: {:?}", index.version_id);
    info!("平台: {:?}", index.platform);

    if !skip_check {
        info!("开始检查旧版文件");

        for m in &index.migrations {
            if let Migrate::Delete(item) = m {
                if item.is_dir == Some(true) {
                    continue;
                }
                info!("校验 {}", item.name.clone().unwrap());
                let (size, hash) = get_file_size_and_hash(root.join(item.clone().name.unwrap()).as_path())?;
                if item.size == Some(size) && item.hash == Some(hash) {
                    continue;
                } else {
                    error!("校验失败 {}", item.name.clone().unwrap());
                    bail!("校验失败 {}", item.name.clone().unwrap());
                }
            }
        }
    }

    info!("开始增量更新");

    for m in &index.migrations {
        match m {
            Migrate::Add(item) => {
                let p = root.join(item.clone().name.unwrap());

                if item.is_dir == Some(true) {
                    info!("创建文件夹 {}", item.name.clone().unwrap());
                    create_dir_all(p)?;
                    continue;
                }

                info!("写入 {}", item.name.clone().unwrap());

                let mut f = File::options()
                    .create(true)
                    .write(true)
                    .open(p)?;
                let zip_file = archive.by_name(format!("assets/{}", item.clone().hash.unwrap()).as_str())?;
                let mut zip_buf = BufReader::new(zip_file);
                loop {
                    let mut buf = vec![0; 1024];
                    let size = zip_buf.read(&mut buf)?;
                    if size == 0 {
                        break;
                    }
                    f.write(&buf[..size])?;
                }
            }
            Migrate::Delete(item) => {
                info!("删除 {}", item.name.clone().unwrap());

                let p = root.join(item.clone().name.unwrap());
                if item.is_dir == Some(true) {
                    fs::remove_dir(p)?;
                } else {
                    fs::remove_file(p)?;
                }
            }
        }
    }

    info!("增量更新成功");

    Ok(())
}