// Copyright (c) 2026 青山公仔
// 轻壳 (Tauri Shell) is licensed under Mulan PSL v2.
// You can use this software according to the terms and conditions of the Mulan PSL v2.
// You may obtain a copy of Mulan PSL v2 at:
//          http://license.coscl.org.cn/MulanPSL2
// THIS SOFTWARE IS PROVIDED ON AN "AS IS" BASIS, WITHOUT WARRANTIES OF ANY KIND,
// EITHER EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO NON-INFRINGEMENT,
// MERCHANTABILITY OR FIT FOR A PARTICULAR PURPOSE.
// See the Mulan PSL v2 for more details.

//! 安装器库入口。同一个 Tauri 壳承载安装向导与卸载向导两种模式，
//! 由 WizardState 区分，前端通过 get_install_config 的 mode 字段分支渲染。

#[cfg(windows)]
pub mod commands;
pub mod config;

/// 向导模式状态：安装，或卸载（携带检测到的卸载现场，未检测到时为 None，
/// UI 显示"未找到安装"提示）
#[cfg(windows)]
#[derive(Clone)]
pub struct WizardState {
    pub(crate) mode: &'static str,
    pub(crate) uninstall_site: Option<commands::uninstall::UninstallSite>,
}

#[cfg(windows)]
mod ipc {
    use std::path::Path;
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::sync::Arc;

    use serde::{Deserialize, Serialize};
    use tauri::{AppHandle, Emitter, State};

    use crate::commands::elevate;
    use crate::commands::install::{self, InstallOptions, InstallPaths};
    use crate::commands::process;
    use crate::commands::progress::{self, InstallProgress};
    use crate::commands::uninstall::{self, UninstallSite};
    use crate::config;
    use crate::WizardState;

    /// 安装进度事件（直接安装与提权轮询转发共用同一契约）
    const PROGRESS_EVENT: &str = "install://progress";

    #[derive(Serialize)]
    #[serde(rename_all = "camelCase")]
    pub struct InstallConfig {
        version: String,
        system_default_dir: String,
        user_default_dir: String,
        /// "install" | "uninstall"，前端据此分支向导流程
        mode: String,
        /// 卸载模式的现场信息；安装模式或未检测到安装时为 None
        uninstall_info: Option<UninstallSite>,
    }

    #[tauri::command]
    pub fn get_install_config(state: State<WizardState>) -> Result<InstallConfig, String> {
        Ok(InstallConfig {
            version: env!("CARGO_PKG_VERSION").into(),
            system_default_dir: InstallPaths::resolve(true)?
                .default_install_dir
                .to_string_lossy()
                .into_owned(),
            user_default_dir: InstallPaths::resolve(false)?
                .default_install_dir
                .to_string_lossy()
                .into_owned(),
            mode: state.mode.into(),
            uninstall_info: state.uninstall_site.clone(),
        })
    }

    /// 目标目录下是否有运行中的进程（主程序/旧卸载器）：
    /// 安装/卸载前端据此弹确认关闭弹窗，未确认不进入后续流程
    #[tauri::command]
    pub fn is_app_running(install_dir: String) -> bool {
        if install_dir.trim().is_empty() {
            return false;
        }
        !process::processes_under(Path::new(&install_dir)).is_empty()
    }

    /// 完成页启动主应用：校验安装目录下主程序存在后以分离进程启动。
    /// UI 进程未提权，主应用天然以普通用户权限运行。
    #[tauri::command]
    pub fn launch_app(install_dir: String) -> Result<(), String> {
        install::validate_install_dir(&install_dir)?;
        let exe = Path::new(&install_dir).join(install::MAIN_EXE_NAME);
        if !exe.is_file() {
            return Err(format!("未找到主程序: {}", exe.display()));
        }
        std::process::Command::new(&exe)
            .current_dir(&install_dir)
            .spawn()
            .map(|_| ())
            .map_err(|e| format!("启动 {} 失败: {e}", config::APP_DISPLAY_NAME))
    }

    /// 完成页引导：深链直达系统设置中本应用的"默认应用"页
    /// （登记在 HKLM/HKCU 的 RegisteredApplications 分别对应 Machine/User 参数）。
    /// Win10/11 的 UserChoice 机制封死程序改默认，设为默认只能由用户在系统 UI 完成
    #[tauri::command]
    pub fn open_default_apps_settings(is_system: bool) -> Result<(), String> {
        use std::os::windows::process::CommandExt;
        const CREATE_NO_WINDOW: u32 = 0x0800_0000;
        let param = if is_system {
            "registeredAppMachine"
        } else {
            "registeredAppUser"
        };
        // URI 可能含空格（应用显示名），std 会自动加引号；空字符串占位 start 的窗口标题位
        let uri = format!(
            "ms-settings:defaultapps?{param}={}",
            install::APP_DISPLAY_NAME
        );
        std::process::Command::new("cmd")
            .args(["/C", "start", "", &uri])
            .creation_flags(CREATE_NO_WINDOW)
            .spawn()
            .map(|_| ())
            .map_err(|e| format!("打开系统设置失败: {e}"))
    }

    #[tauri::command]
    pub async fn start_install(app: AppHandle, options: InstallOptions) -> Result<(), String> {
        // 直接安装与提权安装统一入口校验：非空/绝对路径/无引号，
        // 同时杜绝引号破坏提权命令行的参数注入
        install::validate_install_dir(&options.install_dir)?;
        tauri::async_runtime::spawn_blocking(move || {
            if !options.is_system || elevate::is_elevated() {
                // 用户级安装，或已提权的系统级安装：本进程直接执行，进度走事件
                let sink = |p: &InstallProgress| {
                    let _ = app.emit(PROGRESS_EVENT, p);
                };
                install::run_install(&options, &sink)
            } else {
                install_via_uac(&app, &options)
            }
        })
        .await
        .map_err(|e| format!("安装任务执行失败: {e}"))?
    }

    /// 系统级且未提权：UAC 启动提权子进程执行安装并等待其退出，
    /// 等待期间轮询进度文件转发为 `install://progress` 事件（UI 保持存活）
    fn install_via_uac(app: &AppHandle, options: &InstallOptions) -> Result<(), String> {
        let progress_file = std::env::temp_dir().join(format!(
            "{}-progress-{}.json",
            config::TEMP_PREFIX,
            std::process::id()
        ));
        let args = build_elevated_args(options, &progress_file);

        let (exit_code, final_progress) =
            relaunch_elevated_forwarding_progress(app, &args, &progress_file).map_err(|e| {
                format!("{e}；可取消\"为所有用户安装\"改为用户级安装（无需管理员权限）")
            })?;

        // 轮询可能错过末条进度，按退出码与进度文件终态汇总并补发完成事件
        match final_progress {
            Some(p) if p.error.is_some() => Err(p.error.unwrap()),
            _ if exit_code == 0 => {
                let _ = app.emit(PROGRESS_EVENT, &InstallProgress::finished());
                Ok(())
            }
            _ => Err(format!(
                "提权安装进程异常退出 (exit code: {exit_code})，详见 %TEMP%\\{}.log",
                config::TEMP_PREFIX
            )),
        }
    }

    /// 启动提权子进程执行 args 并阻塞等待退出，期间轮询进度文件转发为
    /// `install://progress` 事件；返回（退出码, 进度文件终态）。
    /// 安装与卸载的提权路径共用，仅终态解释不同。
    fn relaunch_elevated_forwarding_progress(
        app: &AppHandle,
        args: &str,
        progress_file: &Path,
    ) -> Result<(u32, Option<InstallProgress>), String> {
        let _ = std::fs::remove_file(progress_file);

        let stop = Arc::new(AtomicBool::new(false));
        let poller = {
            let stop = Arc::clone(&stop);
            let app = app.clone();
            let path = progress_file.to_path_buf();
            std::thread::spawn(move || {
                let mut last: Option<InstallProgress> = None;
                while !stop.load(Ordering::Relaxed) {
                    if let Some(p) = progress::read_progress_file(&path) {
                        if last.as_ref() != Some(&p) {
                            let _ = app.emit(PROGRESS_EVENT, &p);
                            last = Some(p);
                        }
                    }
                    std::thread::sleep(std::time::Duration::from_millis(200));
                }
            })
        };

        let wait_result = elevate::relaunch_elevated_and_wait(args);
        stop.store(true, Ordering::Relaxed);
        let _ = poller.join();

        let final_progress = progress::read_progress_file(progress_file);
        let _ = std::fs::remove_file(progress_file);

        Ok((wait_result?, final_progress))
    }

    /// 构造提权子进程参数；路径含空格需引号包裹（引号字符已在入口校验拒绝，
    /// 尾随反斜杠会转义引号需先剔除）
    fn build_elevated_args(options: &InstallOptions, progress_file: &Path) -> String {
        let install_dir = options.install_dir.trim_end_matches('\\');
        let mut args = format!(
            "--elevated --install-path=\"{install_dir}\" --progress-file=\"{}\"",
            progress_file.display()
        );
        if options.desktop_shortcut {
            args.push_str(" --desktop-shortcut");
        }
        if options.file_assoc {
            args.push_str(" --file-assoc");
        }
        args
    }

    /// 前端卸载确认页选项
    #[derive(Debug, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct UninstallUiOptions {
        pub remove_user_data: bool,
    }

    #[tauri::command]
    pub async fn start_uninstall(
        app: AppHandle,
        state: State<'_, WizardState>,
        options: UninstallUiOptions,
    ) -> Result<(), String> {
        let site = state
            .uninstall_site
            .clone()
            .ok_or_else(|| format!("未找到 {} 的安装信息，无需卸载", config::APP_DISPLAY_NAME))?;
        install::validate_install_dir(&site.install_dir)?;
        tauri::async_runtime::spawn_blocking(move || {
            if !site.is_system || elevate::is_elevated() {
                // 用户级，或已提权的系统级：本进程直接执行，进度走事件
                let sink = |p: &InstallProgress| {
                    let _ = app.emit(PROGRESS_EVENT, p);
                };
                uninstall::run_uninstall_for_site(&site, options.remove_user_data, &sink)?;
                let _ = app.emit(PROGRESS_EVENT, &InstallProgress::finished_with("卸载完成"));
                Ok(())
            } else {
                uninstall_via_uac(&app, &site, options.remove_user_data)
            }
        })
        .await
        .map_err(|e| format!("卸载任务执行失败: {e}"))?
    }

    /// 系统级且未提权：UAC 启动提权无头 worker（`--uninstall --confirm --detected-*`）
    /// 执行卸载，UI 轮询进度文件转发为同一事件
    fn uninstall_via_uac(
        app: &AppHandle,
        site: &UninstallSite,
        remove_user_data: bool,
    ) -> Result<(), String> {
        let progress_file = std::env::temp_dir().join(format!(
            "{}-progress-{}.json",
            config::TEMP_PREFIX,
            std::process::id()
        ));
        let install_dir = site.install_dir.trim_end_matches('\\');
        let mut args = format!(
            "--uninstall --confirm --detected-install-dir=\"{install_dir}\" {} --progress-file=\"{}\"",
            if site.is_system {
                "--detected-system"
            } else {
                "--detected-user"
            },
            progress_file.display()
        );
        if site.created_desktop_shortcut {
            args.push_str(" --detected-desktop-shortcut");
        }
        if remove_user_data {
            args.push_str(" --remove-user-data");
        }

        let (exit_code, final_progress) =
            relaunch_elevated_forwarding_progress(app, &args, &progress_file)?;

        match final_progress {
            Some(p) if p.error.is_some() => Err(p.error.unwrap()),
            _ if exit_code == 0 => {
                let _ = app.emit(PROGRESS_EVENT, &InstallProgress::finished_with("卸载完成"));
                Ok(())
            }
            _ => Err(format!(
                "提权卸载进程异常退出 (exit code: {exit_code})，详见 %TEMP%\\{}.log",
                config::TEMP_PREFIX
            )),
        }
    }
}

/// 安装向导入口
pub fn run() {
    #[cfg(windows)]
    run_with_state(WizardState {
        mode: "install",
        uninstall_site: None,
    });
    #[cfg(not(windows))]
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

/// 卸载向导入口（site 为 None 时 UI 显示"未找到安装"提示）
#[cfg(windows)]
pub fn run_uninstaller(site: Option<commands::uninstall::UninstallSite>) {
    run_with_state(WizardState {
        mode: "uninstall",
        uninstall_site: site,
    });
}

#[cfg(windows)]
fn run_with_state(state: WizardState) {
    use tauri::Manager;

    let is_uninstall = state.mode == "uninstall";
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .manage(state)
        .setup(move |app| {
            // 卸载模式同步原生窗口标题（任务栏/Alt+Tab 展示）
            if is_uninstall {
                if let Some(win) = app.get_webview_window("main") {
                    let _ = win.set_title(&format!("{} 卸载", config::APP_DISPLAY_NAME));
                }
            }
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            ipc::get_install_config,
            ipc::is_app_running,
            ipc::start_install,
            ipc::launch_app,
            ipc::open_default_apps_settings,
            ipc::start_uninstall
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
