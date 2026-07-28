// Copyright (c) 2026 青山公仔
// 轻壳 (Tauri Shell) is licensed under Mulan PSL v2.
// You can use this software according to the terms and conditions of the Mulan PSL v2.
// You may obtain a copy of Mulan PSL v2 at:
//          http://license.coscl.org.cn/MulanPSL2
// THIS SOFTWARE IS PROVIDED ON AN "AS IS" BASIS, WITHOUT WARRANTIES OF ANY KIND,
// EITHER EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO NON-INFRINGEMENT,
// MERCHANTABILITY OR FIT FOR A PARTICULAR PURPOSE.
// See the Mulan PSL v2 for more details.

//! 安装进度模型与进度文件通道。
//!
//! 直接安装路径由 Tauri 事件承载进度；提权安装的子进程无法向 UI 进程发事件，
//! 改写进度文件（原子替换），UI 进程轮询读取后转发为同一事件。

use std::path::Path;

use serde::{Deserialize, Serialize};

/// 进度上报回调，两种实现：Tauri 事件发射 / 进度文件写入
pub type ProgressSink<'a> = &'a dyn Fn(&InstallProgress);

/// 安装进度。字段名 camelCase：与前端 `install://progress` 事件负载契约一致
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InstallProgress {
    /// 当前步骤标识（extract / copy / register / shortcut / done）
    pub step: String,
    pub percent: u8,
    pub message: String,
    pub done: bool,
    pub error: Option<String>,
}

impl InstallProgress {
    pub fn running(step: &str, percent: u8, message: String) -> Self {
        Self {
            step: step.into(),
            percent,
            message,
            done: false,
            error: None,
        }
    }

    pub fn finished() -> Self {
        Self::finished_with("安装完成")
    }

    /// 完成态（自定义文案，卸载流程复用同一进度模型）
    pub fn finished_with(message: &str) -> Self {
        Self {
            step: "done".into(),
            percent: 100,
            message: message.into(),
            done: true,
            error: None,
        }
    }

    pub fn failed(error: String) -> Self {
        Self {
            step: "error".into(),
            percent: 0,
            message: "安装失败".into(),
            done: true,
            error: Some(error),
        }
    }
}

/// 原子写入进度文件：先写临时文件再 rename，防止轮询方读到半截 JSON
pub fn write_progress_file(path: &Path, progress: &InstallProgress) -> Result<(), String> {
    let json = serde_json::to_string(progress).map_err(|e| format!("序列化进度失败: {e}"))?;
    let tmp = path.with_extension("json.tmp");
    std::fs::write(&tmp, json).map_err(|e| format!("写入进度临时文件失败: {e}"))?;
    std::fs::rename(&tmp, path).map_err(|e| format!("替换进度文件失败: {e}"))
}

/// 读取进度文件。文件缺失或损坏返回 None（轮询方继续等待下一轮）
pub fn read_progress_file(path: &Path) -> Option<InstallProgress> {
    let json = std::fs::read_to_string(path).ok()?;
    serde_json::from_str(&json).ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn progress_file_roundtrip() {
        let path = std::env::temp_dir().join(format!(
            "my-app-progress-test-{}.json",
            std::process::id()
        ));
        let progress = InstallProgress::running("extract", 42, "正在解压...".into());

        write_progress_file(&path, &progress).unwrap();
        assert_eq!(read_progress_file(&path), Some(progress));

        // 覆盖写入取最新值
        let done = InstallProgress::finished();
        write_progress_file(&path, &done).unwrap();
        assert_eq!(read_progress_file(&path), Some(done));

        std::fs::remove_file(&path).unwrap();
    }

    #[test]
    fn read_progress_file_tolerates_missing_and_corrupt() {
        let missing = std::env::temp_dir().join("my-app-progress-missing.json");
        assert_eq!(read_progress_file(&missing), None);

        let corrupt = std::env::temp_dir().join(format!(
            "my-app-progress-corrupt-{}.json",
            std::process::id()
        ));
        std::fs::write(&corrupt, "not json{{").unwrap();
        assert_eq!(read_progress_file(&corrupt), None);
        std::fs::remove_file(&corrupt).unwrap();
    }
}
