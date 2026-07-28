// Copyright (c) 2026 青山公仔
// 轻壳 (Tauri Shell) is licensed under Mulan PSL v2.
// You can use this software according to the terms and conditions of the Mulan PSL v2.
// You may obtain a copy of Mulan PSL v2 at:
//          http://license.coscl.org.cn/MulanPSL2
// THIS SOFTWARE IS PROVIDED ON AN "AS IS" BASIS, WITHOUT WARRANTIES OF ANY KIND,
// EITHER EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO NON-INFRINGEMENT,
// MERCHANTABILITY OR FIT FOR A PARTICULAR PURPOSE.
// See the Mulan PSL v2 for more details.

//! 终止安装目录下运行中的进程：卸载前关闭主程序，释放 exe 文件锁，
//! 保证主程序可执行文件与安装目录能被完整删除。

use std::path::Path;

use windows::core::PWSTR;
use windows::Win32::Foundation::CloseHandle;
use windows::Win32::System::Diagnostics::ToolHelp::{
    CreateToolhelp32Snapshot, Process32FirstW, Process32NextW, PROCESSENTRY32W,
    TH32CS_SNAPPROCESS,
};
use windows::Win32::System::Threading::{
    OpenProcess, QueryFullProcessImageNameW, TerminateProcess, WaitForSingleObject,
    PROCESS_NAME_WIN32, PROCESS_QUERY_LIMITED_INFORMATION, PROCESS_SYNCHRONIZE,
    PROCESS_TERMINATE,
};

/// 终止后等待进程退出的上限
const WAIT_EXIT_MS: u32 = 5_000;

/// 目录前缀（小写 + 尾部分隔符），Windows 路径大小写不敏感
fn dir_prefix(dir: &Path) -> String {
    let mut s = dir.to_string_lossy().to_lowercase();
    if !s.ends_with('\\') && !s.ends_with('/') {
        s.push('\\');
    }
    s
}

/// path 是否位于 dir 目录下（小写化前缀比较，与进程匹配同一标准，
/// main.rs 判断卸载器是否在安装目录内时同样使用）
pub fn path_is_under(path: &Path, dir: &Path) -> bool {
    path.to_string_lossy()
        .to_lowercase()
        .starts_with(&dir_prefix(dir))
}

/// 查询进程镜像完整路径；无权限（如系统进程）返回 None
fn process_image_path(pid: u32) -> Option<String> {
    unsafe {
        let handle = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, pid).ok()?;
        let mut buf = [0u16; 1024];
        let mut len = buf.len() as u32;
        let result =
            QueryFullProcessImageNameW(handle, PROCESS_NAME_WIN32, PWSTR(buf.as_mut_ptr()), &mut len);
        let _ = CloseHandle(handle);
        result.ok()?;
        Some(String::from_utf16_lossy(&buf[..len as usize]))
    }
}

/// 枚举镜像路径位于 dir 下的进程 pid（不含当前进程自身）
pub fn processes_under(dir: &Path) -> Vec<u32> {
    let prefix = dir_prefix(dir);
    let current_pid = std::process::id();
    let mut pids = Vec::new();
    unsafe {
        let Ok(snapshot) = CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0) else {
            return pids;
        };
        let mut entry = PROCESSENTRY32W {
            dwSize: std::mem::size_of::<PROCESSENTRY32W>() as u32,
            ..Default::default()
        };
        if Process32FirstW(snapshot, &mut entry).is_ok() {
            loop {
                let pid = entry.th32ProcessID;
                if pid != current_pid {
                    if let Some(path) = process_image_path(pid) {
                        if path.to_lowercase().starts_with(&prefix) {
                            pids.push(pid);
                        }
                    }
                }
                if Process32NextW(snapshot, &mut entry).is_err() {
                    break;
                }
            }
        }
        let _ = CloseHandle(snapshot);
    }
    pids
}

/// 强制终止 dir 下运行中的进程并等待退出，返回错误列表（空 = 全部成功）。
/// 卸载场景使用 TerminateProcess 强制结束（主应用有自动保存，确认页已提示）。
pub fn kill_processes_under(dir: &Path) -> Vec<String> {
    let mut errors = Vec::new();
    for pid in processes_under(dir) {
        unsafe {
            // 枚举与终止之间进程可能已自行退出，OpenProcess 失败不计错误
            //（若确为权限问题，后续文件删除会以占用超时形式暴露）
            let Ok(handle) = OpenProcess(PROCESS_TERMINATE | PROCESS_SYNCHRONIZE, false, pid)
            else {
                continue;
            };
            if let Err(e) = TerminateProcess(handle, 1) {
                errors.push(format!("终止进程 {pid} 失败: {e}"));
            } else {
                // 等待退出，确保后续文件删除时句柄已释放
                WaitForSingleObject(handle, WAIT_EXIT_MS);
            }
            let _ = CloseHandle(handle);
        }
    }
    errors
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn processes_under_empty_dir_finds_nothing() {
        let dir = std::env::temp_dir().join(format!(
            "my-app-process-test-{}",
            std::process::id()
        ));
        std::fs::create_dir_all(&dir).unwrap();

        assert!(processes_under(&dir).is_empty());
        assert!(kill_processes_under(&dir).is_empty());

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn processes_under_excludes_self() {
        // 用当前测试 exe 所在目录验证：枚举结果不包含自身 pid
        let exe_dir = std::env::current_exe().unwrap().parent().unwrap().to_path_buf();
        let pids = processes_under(&exe_dir);
        assert!(!pids.contains(&std::process::id()));
    }

    #[test]
    fn dir_prefix_is_lowercase_with_separator() {
        assert_eq!(dir_prefix(Path::new(r"C:\Fake\My App")), r"c:\fake\my app\");
        assert_eq!(dir_prefix(Path::new(r"C:\Fake\My App\")), r"c:\fake\my app\");
    }

    #[test]
    fn path_is_under_ignores_case_and_requires_boundary() {
        let dir = Path::new(r"C:\Program Files\My App");
        assert!(path_is_under(
            Path::new(r"c:\program files\my app\my-app-wizard.exe"),
            dir
        ));
        // 目录边界：同前缀的兄弟目录不算在内
        assert!(!path_is_under(
            Path::new(r"C:\Program Files\My App 2\app.exe"),
            dir
        ));
    }
}
