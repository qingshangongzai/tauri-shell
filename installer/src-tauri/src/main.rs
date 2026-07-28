// Copyright (c) 2026 青山公仔
// 轻壳 (Tauri Shell) is licensed under Mulan PSL v2.
// You can use this software according to the terms and conditions of the Mulan PSL v2.
// You may obtain a copy of Mulan PSL v2 at:
//          http://license.coscl.org.cn/MulanPSL2
// THIS SOFTWARE IS PROVIDED ON AN "AS IS" BASIS, WITHOUT WARRANTIES OF ANY KIND,
// EITHER EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO NON-INFRINGEMENT,
// MERCHANTABILITY OR FIT FOR A PARTICULAR PURPOSE.
// See the Mulan PSL v2 for more details.

// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.iter().any(|a| a == "--uninstall") {
        if args.iter().any(|a| a == "--confirm") {
            // 无头卸载：提权 worker / CLI 直接执行
            run_uninstall_headless(&args);
        } else {
            // 卸载向导 UI（控制面板 UninstallString 走此路径）
            run_uninstall_ui(&args);
        }
    } else if args.iter().any(|a| a == "--elevated") {
        // 管理员模式：UAC 提权后重启进入，执行实际系统级安装
        run_elevated_mode(&args);
    } else {
        installer_lib::run();
    }
}

/// release 构建为 windows 子系统（无控制台），附加父进程控制台使 println! 可见；
/// 非控制台启动时附加失败属预期，忽略即可
#[cfg(windows)]
fn attach_parent_console() {
    use windows::Win32::System::Console::{AttachConsole, ATTACH_PARENT_PROCESS};
    unsafe {
        let _ = AttachConsole(ATTACH_PARENT_PROCESS);
    }
}

/// 卸载向导 UI 入口：解析/检测卸载现场后启动 Tauri 卸载向导。
/// 卸载器位于安装目录内时先自复制到 %TEMP% 重启（UI 进程不锁自身 exe，
/// 后续删除安装目录才能连同卸载器一并移除）。
#[cfg(windows)]
fn run_uninstall_ui(args: &[String]) {
    use installer_lib::commands::uninstall::{detect_install, parse_detected_args};

    attach_parent_console();

    let detected = match parse_detected_args(args) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("参数错误: {e}");
            std::process::exit(1);
        }
    };
    let from_temp_copy = detected.is_some();
    let site = match detected {
        Some(d) => Some(d.to_site()),
        None => detect_install(),
    };

    if !from_temp_copy {
        if let Some(site) = &site {
            if exe_inside_install_dir(&site.install_dir) {
                match relaunch_from_temp_copy(site, false, false) {
                    Ok(()) => return, // 临时副本接管 UI，当前进程退出释放文件锁
                    Err(e) => eprintln!(
                        "启动临时副本失败（{e}），继续在安装目录内运行，卸载器自身可能残留。"
                    ),
                }
            }
        }
    }

    installer_lib::run_uninstaller(site);
}

/// 无头卸载：自动检测或按透传参数重建现场后直接执行。
/// 进度写 `--progress-file=`（提权 worker）或控制台输出。
#[cfg(windows)]
fn run_uninstall_headless(args: &[String]) {
    use std::path::PathBuf;

    use installer_lib::commands::elevate::{is_elevated, relaunch_elevated};
    use installer_lib::commands::progress::{self, InstallProgress};
    use installer_lib::commands::uninstall::{
        detect_install, parse_detected_args, run_uninstall_for_site,
    };

    attach_parent_console();

    let detected = match parse_detected_args(args) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("参数错误: {e}");
            std::process::exit(1);
        }
    };
    let remove_user_data = args.iter().any(|a| a == "--remove-user-data");
    let site = match &detected {
        Some(d) => d.to_site(),
        None => {
            let Some(found) = detect_install() else {
                println!(
                    "未找到 {} 的安装信息（install-info.json / HKCU / HKLM），无需卸载。",
                    installer_lib::config::APP_DISPLAY_NAME
                );
                return;
            };
            found
        }
    };

    // 系统级卸载需要管理员权限：未提权时携带完整现场参数 UAC 重启自身
    if site.is_system && !is_elevated() {
        println!("系统级安装需要管理员权限，正在请求 UAC 提权...");
        let mut relaunch_args = format!(
            "--uninstall --confirm --detected-install-dir=\"{}\" --detected-system",
            site.install_dir.trim_end_matches('\\')
        );
        if site.created_desktop_shortcut {
            relaunch_args.push_str(" --detected-desktop-shortcut");
        }
        if remove_user_data {
            relaunch_args.push_str(" --remove-user-data");
        }
        if let Err(e) = relaunch_elevated(&relaunch_args) {
            eprintln!("{e}");
            std::process::exit(1);
        }
        return; // 提权进程接管卸载，当前进程退出
    }

    // 卸载器位于待删除的安装目录内：自复制到 %TEMP% 重启
    //（提权已在上方完成，spawn 的子进程自动继承提权），当前进程退出释放文件锁
    if exe_inside_install_dir(&site.install_dir) {
        match relaunch_from_temp_copy(&site, true, remove_user_data) {
            Ok(()) => {
                println!("卸载器位于安装目录内，已从临时副本继续执行卸载。");
                return;
            }
            Err(e) => {
                eprintln!("启动临时副本失败（{e}），继续就地卸载，卸载器自身可能残留。");
            }
        }
    }

    // 进度通道：提权 worker 写进度文件（UI 轮询转发），CLI 场景控制台输出
    let progress_file: Option<PathBuf> = args
        .iter()
        .find_map(|a| a.strip_prefix("--progress-file="))
        .map(PathBuf::from);
    let sink = |p: &InstallProgress| {
        if let Some(path) = &progress_file {
            let _ = progress::write_progress_file(path, p);
        } else {
            println!("[{:3}%] {}", p.percent, p.message);
        }
    };

    match run_uninstall_for_site(&site, remove_user_data, &sink) {
        Ok(()) => {
            sink(&InstallProgress::finished_with("卸载完成"));
            println!("卸载完成。");
        }
        Err(e) => {
            sink(&InstallProgress::failed(e.clone()));
            eprintln!("卸载完成，但存在以下错误：\n{e}");
            std::process::exit(1);
        }
    }
}

/// 当前 exe 是否位于安装目录内（小写化前缀比较，Windows 路径大小写不敏感；
/// 空路径保护：注册表回退检测的 InstallLocation 可能为空）
#[cfg(windows)]
fn exe_inside_install_dir(install_dir: &str) -> bool {
    use installer_lib::commands::process::path_is_under;

    !install_dir.is_empty()
        && std::env::current_exe()
            .ok()
            .is_some_and(|exe| path_is_under(&exe, std::path::Path::new(install_dir)))
}

#[cfg(windows)]
fn run_elevated_mode(args: &[String]) {
    use installer_lib::commands::install::run_elevated_install;

    // 经 ShellExecuteW("runas") 启动的提权进程不继承原进程控制台，attach 仅对
    // 直接从（提权）终端手动运行时生效；真实提权场景的错误另落日志可观测
    attach_parent_console();

    if let Err(e) = run_elevated_install(args) {
        eprintln!("提权安装失败: {e}");
        log_elevated_error(&e);
        std::process::exit(1);
    }
}

/// 将卸载器自身复制到 %TEMP% 并携带 --detected-* 透传参数重启（confirm=false
/// 为 UI 模式，true 为无头执行），解决卸载器位于安装目录内无法删除运行中自身。
/// 临时副本执行完毕后自身残留于 %TEMP%（可接受，与 NSIS 行为一致）。
#[cfg(windows)]
fn relaunch_from_temp_copy(
    site: &installer_lib::commands::uninstall::UninstallSite,
    confirm: bool,
    remove_user_data: bool,
) -> Result<(), String> {
    use installer_lib::commands::install::validate_install_dir;

    // 路径将作为 --detected-install-dir= 参数透传，提前校验避免副本侧拒绝
    validate_install_dir(&site.install_dir)?;
    let current_exe =
        std::env::current_exe().map_err(|e| format!("无法获取卸载器路径: {e}"))?;
    let temp_exe = std::env::temp_dir().join(format!(
        "{}-uninstall-{}.exe",
        installer_lib::config::TEMP_PREFIX,
        std::process::id()
    ));
    std::fs::copy(&current_exe, &temp_exe)
        .map_err(|e| format!("复制卸载器到临时目录失败: {e}"))?;

    let mut cmd = std::process::Command::new(&temp_exe);
    cmd.arg("--uninstall");
    if confirm {
        cmd.arg("--confirm");
    }
    cmd.arg(format!("--detected-install-dir={}", site.install_dir))
        .arg(if site.is_system {
            "--detected-system"
        } else {
            "--detected-user"
        });
    if site.created_desktop_shortcut {
        cmd.arg("--detected-desktop-shortcut");
    }
    if remove_user_data {
        cmd.arg("--remove-user-data");
    }
    cmd.spawn()
        .map(|_| ())
        .map_err(|e| format!("启动临时副本失败: {e}"))
}

/// 提权安装错误追加写入 %TEMP%\{TEMP_PREFIX}.log，日志写入失败不影响退出码
#[cfg(windows)]
fn log_elevated_error(msg: &str) {
    use std::io::Write;

    let log_path = std::env::temp_dir().join(format!(
        "{}.log",
        installer_lib::config::TEMP_PREFIX
    ));
    if let Ok(mut file) = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&log_path)
    {
        let _ = writeln!(file, "[elevated] 提权安装失败: {msg}");
    }
}

#[cfg(not(windows))]
fn run_uninstall_ui(_args: &[String]) {
    eprintln!("卸载模式仅支持 Windows。");
}

#[cfg(not(windows))]
fn run_uninstall_headless(_args: &[String]) {
    eprintln!("卸载模式仅支持 Windows。");
}

#[cfg(not(windows))]
fn run_elevated_mode(_args: &[String]) {
    eprintln!("提权安装模式仅支持 Windows。");
}
