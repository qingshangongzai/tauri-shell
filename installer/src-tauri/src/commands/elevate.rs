// Copyright (c) 2026 青山公仔
// 轻壳 (Tauri Shell) is licensed under Mulan PSL v2.
// You can use this software according to the terms and conditions of the Mulan PSL v2.
// You may obtain a copy of Mulan PSL v2 at:
//          http://license.coscl.org.cn/MulanPSL2
// THIS SOFTWARE IS PROVIDED ON AN "AS IS" BASIS, WITHOUT WARRANTIES OF ANY KIND,
// EITHER EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO NON-INFRINGEMENT,
// MERCHANTABILITY OR FIT FOR A PARTICULAR PURPOSE.
// See the Mulan PSL v2 for more details.

//! UAC 提权：提权状态检测、`ShellExecuteW("runas")` 重启自身、
//! 及启动提权子进程并等待退出（安装向导 UI 存活场景）。

use windows::core::PCWSTR;
use windows::Win32::Foundation::{CloseHandle, GetLastError, ERROR_CANCELLED, HANDLE};
use windows::Win32::Security::{
    GetTokenInformation, TokenElevation, TOKEN_ELEVATION, TOKEN_QUERY,
};
use windows::Win32::System::Threading::{
    GetCurrentProcess, GetExitCodeProcess, OpenProcessToken, WaitForSingleObject, INFINITE,
};
use windows::Win32::UI::Shell::{
    ShellExecuteExW, ShellExecuteW, SEE_MASK_NOCLOSEPROCESS, SHELLEXECUTEINFOW,
};
use windows::Win32::UI::WindowsAndMessaging::SW_SHOWNORMAL;

use super::wide;

/// 检测当前进程是否已提权（TokenElevation）。查询失败时保守返回 false。
pub fn is_elevated() -> bool {
    unsafe {
        let mut token = HANDLE::default();
        if OpenProcessToken(GetCurrentProcess(), TOKEN_QUERY, &mut token).is_err() {
            return false;
        }
        let mut elevation = TOKEN_ELEVATION::default();
        let mut returned = 0u32;
        let ok = GetTokenInformation(
            token,
            TokenElevation,
            Some(&mut elevation as *mut _ as *mut _),
            std::mem::size_of::<TOKEN_ELEVATION>() as u32,
            &mut returned,
        )
        .is_ok();
        let _ = CloseHandle(token);
        ok && elevation.TokenIsElevated != 0
    }
}

/// 以管理员身份重新启动当前 exe（触发 UAC 弹窗）。
/// 成功返回 Ok 后由调用方退出当前进程；用户拒绝 UAC 时返回专门的错误信息。
pub fn relaunch_elevated(args: &str) -> Result<(), String> {
    let exe_path = std::env::current_exe()
        .map_err(|e| format!("无法获取可执行文件路径: {e}"))?
        .to_string_lossy()
        .into_owned();

    // PCWSTR 要求调用期间缓冲区保持存活
    let verb_w = wide("runas");
    let exe_w = wide(&exe_path);
    let args_w = wide(args);

    unsafe {
        let result = ShellExecuteW(
            None,
            PCWSTR(verb_w.as_ptr()),
            PCWSTR(exe_w.as_ptr()),
            PCWSTR(args_w.as_ptr()),
            None,
            SW_SHOWNORMAL,
        );

        // 返回值 >32 表示成功（ShellExecuteW 历史约定）
        if result.0 as isize > 32 {
            return Ok(());
        }
        let last_error = GetLastError();
        if last_error == ERROR_CANCELLED {
            Err("用户拒绝了 UAC 提权请求".into())
        } else {
            Err(format!(
                "UAC 提权失败 (code: {}, last error: {})",
                result.0 as isize,
                last_error.0
            ))
        }
    }
}

/// 以管理员身份启动当前 exe 并阻塞等待其退出，返回子进程退出码。
/// 安装向导 UI 保持存活，等待期间由调用方轮询进度文件呈现真实进度。
pub fn relaunch_elevated_and_wait(args: &str) -> Result<u32, String> {
    let exe_path = std::env::current_exe()
        .map_err(|e| format!("无法获取可执行文件路径: {e}"))?
        .to_string_lossy()
        .into_owned();

    // PCWSTR 要求调用期间缓冲区保持存活
    let verb_w = wide("runas");
    let exe_w = wide(&exe_path);
    let args_w = wide(args);

    // SEE_MASK_NOCLOSEPROCESS：要求返回 hProcess 以供等待
    let mut info = SHELLEXECUTEINFOW {
        cbSize: std::mem::size_of::<SHELLEXECUTEINFOW>() as u32,
        fMask: SEE_MASK_NOCLOSEPROCESS,
        lpVerb: PCWSTR(verb_w.as_ptr()),
        lpFile: PCWSTR(exe_w.as_ptr()),
        lpParameters: PCWSTR(args_w.as_ptr()),
        nShow: SW_SHOWNORMAL.0,
        ..Default::default()
    };

    unsafe {
        if let Err(e) = ShellExecuteExW(&mut info) {
            let last_error = GetLastError();
            return if last_error == ERROR_CANCELLED {
                Err("用户拒绝了 UAC 提权请求".into())
            } else {
                Err(format!("UAC 提权失败: {e} (last error: {})", last_error.0))
            };
        }
        if info.hProcess.is_invalid() {
            return Err("UAC 提权成功但未获得子进程句柄".into());
        }

        WaitForSingleObject(info.hProcess, INFINITE);
        let mut exit_code = 0u32;
        let result = GetExitCodeProcess(info.hProcess, &mut exit_code);
        let _ = CloseHandle(info.hProcess);
        result.map_err(|e| format!("获取提权进程退出码失败: {e}"))?;
        Ok(exit_code)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn is_elevated_returns_without_panic() {
        // UAC 弹窗流程无法自动化，仅验证提权状态查询可正常调用；
        // 测试环境下通常为非提权进程，但不断言具体值
        let _elevated: bool = is_elevated();
    }
}
