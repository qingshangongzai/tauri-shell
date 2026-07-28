// Copyright (c) 2026 青山公仔
// 轻壳 (Tauri Shell) is licensed under Mulan PSL v2.
// You can use this software according to the terms and conditions of the Mulan PSL v2.
// You may obtain a copy of Mulan PSL v2 at:
//          http://license.coscl.org.cn/MulanPSL2
// THIS SOFTWARE IS PROVIDED ON AN "AS IS" BASIS, WITHOUT WARRANTIES OF ANY KIND,
// EITHER EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO NON-INFRINGEMENT,
// MERCHANTABILITY OR FIT FOR A PARTICULAR PURPOSE.
// See the Mulan PSL v2 for more details.

//! 嵌入的主应用 zip 包访问与解压。
//!
//! bundle 由 build.rs 复制到 OUT_DIR 后编译期嵌入；开发/测试期缺包时为
//! 零字节占位，运行时报错提示先执行打包脚本。

use std::io::{Cursor, Read};
use std::path::Path;

use zip::ZipArchive;

/// 编译期嵌入的主应用包（可能为零字节占位）
static EMBEDDED_BUNDLE: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/app-bundle.zip"));

/// 返回嵌入的主应用包字节；未嵌入（占位）时报错
pub fn embedded_bundle() -> Result<&'static [u8], String> {
    if EMBEDDED_BUNDLE.is_empty() {
        Err("安装包未嵌入，请先执行打包脚本生成 app-bundle.zip 后重新编译安装器".into())
    } else {
        Ok(EMBEDDED_BUNDLE)
    }
}

/// 将 zip 字节解压到目标目录，返回解压总字节数（供 EstimatedSize 估算）。
/// `on_entry(已完成条目数, 总条目数)` 在每个条目解压后回调。
pub fn extract_zip(
    bytes: &[u8],
    target_dir: &Path,
    on_entry: &dyn Fn(usize, usize),
) -> Result<u64, String> {
    let mut archive =
        ZipArchive::new(Cursor::new(bytes)).map_err(|e| format!("读取安装包失败: {e}"))?;
    std::fs::create_dir_all(target_dir).map_err(|e| format!("创建安装目录失败: {e}"))?;

    let total = archive.len();
    let mut extracted_bytes = 0u64;
    for i in 0..total {
        let mut entry = archive
            .by_index(i)
            .map_err(|e| format!("读取安装包条目失败: {e}"))?;
        // 防 zip-slip：拒绝解压出目标目录外的路径
        let Some(relative) = entry.enclosed_name() else {
            return Err(format!("安装包含非法路径条目: {}", entry.name()));
        };
        let dest = target_dir.join(relative);

        if entry.is_dir() {
            std::fs::create_dir_all(&dest)
                .map_err(|e| format!("创建目录 {} 失败: {e}", dest.display()))?;
        } else {
            if let Some(parent) = dest.parent() {
                std::fs::create_dir_all(parent)
                    .map_err(|e| format!("创建目录 {} 失败: {e}", parent.display()))?;
            }
            let mut buf = Vec::with_capacity(entry.size() as usize);
            entry
                .read_to_end(&mut buf)
                .map_err(|e| format!("解压 {} 失败: {e}", dest.display()))?;
            std::fs::write(&dest, &buf)
                .map_err(|e| format!("写入 {} 失败: {e}", dest.display()))?;
            extracted_bytes += buf.len() as u64;
        }
        on_entry(i + 1, total);
    }
    Ok(extracted_bytes)
}

/// 测试辅助：现场构造测试 zip（不依赖真实主程序包），install.rs 测试同样使用
#[cfg(test)]
pub(crate) mod test_util {
    use std::io::{Cursor, Write};
    use zip::write::SimpleFileOptions;
    use zip::ZipWriter;

    pub fn build_test_zip(entries: &[(&str, &[u8])]) -> Vec<u8> {
        let mut writer = ZipWriter::new(Cursor::new(Vec::new()));
        let options = SimpleFileOptions::default();
        for (name, data) in entries {
            if name.ends_with('/') {
                writer
                    .add_directory(name.trim_end_matches('/'), options)
                    .unwrap();
            } else {
                writer.start_file(*name, options).unwrap();
                writer.write_all(data).unwrap();
            }
        }
        writer.finish().unwrap().into_inner()
    }
}

#[cfg(test)]
mod tests {
    use super::test_util::build_test_zip;
    use super::*;

    #[test]
    fn extract_zip_roundtrip_with_progress() {
        let zip = build_test_zip(&[
            ("my-app.exe", b"fake exe".as_slice()),
            ("resources/", b"".as_slice()),
            ("resources/app.json", b"{}".as_slice()),
        ]);
        let target = std::env::temp_dir().join(format!(
            "my-app-extract-test-{}",
            std::process::id()
        ));

        let mut reported = Vec::new();
        let extracted = {
            let reported_cell = std::cell::RefCell::new(&mut reported);
            extract_zip(&zip, &target, &|done, total| {
                reported_cell.borrow_mut().push((done, total));
            })
            .unwrap()
        };

        assert_eq!(
            std::fs::read(target.join("my-app.exe")).unwrap(),
            b"fake exe"
        );
        assert_eq!(
            std::fs::read(target.join("resources/app.json")).unwrap(),
            b"{}"
        );
        // 总字节数 = 两个文件内容之和；目录条目不计
        assert_eq!(extracted, 10);
        assert_eq!(reported, vec![(1, 3), (2, 3), (3, 3)]);

        std::fs::remove_dir_all(&target).unwrap();
    }

    #[test]
    fn extract_zip_rejects_zip_slip() {
        let zip = build_test_zip(&[("../escape.txt", b"evil".as_slice())]);
        let target = std::env::temp_dir().join(format!(
            "my-app-zipslip-test-{}",
            std::process::id()
        ));

        let err = extract_zip(&zip, &target, &|_, _| {}).unwrap_err();
        assert!(err.contains("非法路径"), "意外错误: {err}");
        assert!(!std::env::temp_dir().join("escape.txt").exists());

        let _ = std::fs::remove_dir_all(&target);
    }

    #[test]
    fn extract_zip_rejects_invalid_bytes() {
        let target = std::env::temp_dir().join("my-app-badzip-test");
        assert!(extract_zip(b"not a zip", &target, &|_, _| {}).is_err());
    }
}
