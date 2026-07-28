// Copyright (c) 2026 青山公仔
// 轻壳 (Tauri Shell) is licensed under Mulan PSL v2.
// You can use this software according to the terms and conditions of the Mulan PSL v2.
// You may obtain a copy of Mulan PSL v2 at:
//          http://license.coscl.org.cn/MulanPSL2
// THIS SOFTWARE IS PROVIDED ON AN "AS IS" BASIS, WITHOUT WARRANTIES OF ANY KIND,
// EITHER EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO NON-INFRINGEMENT,
// MERCHANTABILITY OR FIT FOR A PARTICULAR PURPOSE.
// See the Mulan PSL v2 for more details.

//! .lnk 快捷方式创建（COM `IShellLinkW` 接口）。

use std::path::Path;

use windows::core::{Interface, PCWSTR};
use windows::Win32::Foundation::RPC_E_CHANGED_MODE;
use windows::Win32::System::Com::{
    CoCreateInstance, CoInitializeEx, CoUninitialize, IPersistFile, CLSCTX_INPROC_SERVER,
    COINIT_APARTMENTTHREADED,
};
use windows::Win32::UI::Shell::{IShellLinkW, ShellLink};

use super::wide;

/// COM 初始化 RAII guard：仅当本次 CoInitializeEx 成功时才在 Drop 中配对 CoUninitialize
struct ComGuard {
    initialized: bool,
}

impl ComGuard {
    fn init() -> Result<Self, String> {
        // S_OK/S_FALSE 均为成功且需配对释放；RPC_E_CHANGED_MODE 表示线程已按其他模式
        // 初始化，可继续使用但不得由本 guard 释放
        let hr = unsafe { CoInitializeEx(None, COINIT_APARTMENTTHREADED) };
        if hr.is_err() && hr != RPC_E_CHANGED_MODE {
            return Err(format!("COM 初始化失败: {hr}"));
        }
        Ok(Self {
            initialized: hr.is_ok(),
        })
    }
}

impl Drop for ComGuard {
    fn drop(&mut self) {
        if self.initialized {
            unsafe { CoUninitialize() };
        }
    }
}

/// 创建 .lnk 快捷方式。COM 初始化失败时返回 Err，由调用方决定是否降级跳过。
pub fn create_shortcut(
    target_path: &str,
    shortcut_path: &str,
    description: &str,
    working_dir: &str,
    icon_path: &str,
) -> Result<(), String> {
    if let Some(parent) = Path::new(shortcut_path).parent() {
        std::fs::create_dir_all(parent).map_err(|e| format!("创建快捷方式目录失败: {e}"))?;
    }

    let _com = ComGuard::init()?;

    unsafe {
        let shell_link: IShellLinkW = CoCreateInstance(&ShellLink, None, CLSCTX_INPROC_SERVER)
            .map_err(|e| format!("创建 IShellLink 实例失败: {e}"))?;

        let target_w = wide(target_path);
        let working_dir_w = wide(working_dir);
        let description_w = wide(description);
        let icon_w = wide(icon_path);

        shell_link
            .SetPath(PCWSTR(target_w.as_ptr()))
            .map_err(|e| format!("设置快捷方式目标失败: {e}"))?;
        shell_link
            .SetWorkingDirectory(PCWSTR(working_dir_w.as_ptr()))
            .map_err(|e| format!("设置工作目录失败: {e}"))?;
        shell_link
            .SetDescription(PCWSTR(description_w.as_ptr()))
            .map_err(|e| format!("设置描述失败: {e}"))?;
        shell_link
            .SetIconLocation(PCWSTR(icon_w.as_ptr()), 0)
            .map_err(|e| format!("设置图标失败: {e}"))?;

        let persist_file: IPersistFile = shell_link
            .cast()
            .map_err(|e| format!("获取 IPersistFile 失败: {e}"))?;
        let shortcut_w = wide(shortcut_path);
        persist_file
            .Save(PCWSTR(shortcut_w.as_ptr()), true)
            .map_err(|e| format!("保存快捷方式失败: {e}"))?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_shortcut_in_temp_dir() {
        let target = std::env::current_exe().unwrap();
        let target_dir = target.parent().unwrap();
        let lnk_path = std::env::temp_dir().join(format!(
            "my-app-wizard-test-{}.lnk",
            std::process::id()
        ));

        create_shortcut(
            &target.to_string_lossy(),
            &lnk_path.to_string_lossy(),
            "My App 测试快捷方式",
            &target_dir.to_string_lossy(),
            &target.to_string_lossy(),
        )
        .unwrap();

        assert!(lnk_path.exists());
        // .lnk 是二进制文件且以固定头开始（0x4C 'L'）
        let bytes = std::fs::read(&lnk_path).unwrap();
        assert_eq!(bytes[0], 0x4C);

        std::fs::remove_file(&lnk_path).unwrap();
    }

    #[test]
    fn create_shortcut_creates_parent_dirs() {
        let target = std::env::current_exe().unwrap();
        let base = std::env::temp_dir().join(format!(
            "my-app-wizard-test-dir-{}",
            std::process::id()
        ));
        let lnk_path = base.join("嵌套目录").join("My App.lnk");

        create_shortcut(
            &target.to_string_lossy(),
            &lnk_path.to_string_lossy(),
            "",
            &target.parent().unwrap().to_string_lossy(),
            &target.to_string_lossy(),
        )
        .unwrap();

        assert!(lnk_path.exists());
        std::fs::remove_dir_all(&base).unwrap();
    }
}
