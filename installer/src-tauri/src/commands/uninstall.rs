// Copyright (c) 2026 青山公仔
// 轻壳 (Tauri Shell) is licensed under Mulan PSL v2.
// You can use this software according to the terms and conditions of the Mulan PSL v2.
// You may obtain a copy of Mulan PSL v2 at:
//          http://license.coscl.org.cn/MulanPSL2
// THIS SOFTWARE IS PROVIDED ON AN "AS IS" BASIS, WITHOUT WARRANTIES OF ANY KIND,
// EITHER EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO NON-INFRINGEMENT,
// MERCHANTABILITY OR FIT FOR A PARTICULAR PURPOSE.
// See the Mulan PSL v2 for more details.

//! 卸载编排：关闭主程序后按顺序清理快捷方式、文件关联、注册表卸载信息、
//! 安装目录与（可选）用户数据。每步失败不中断后续步骤，错误收集后汇总返回；
//! 进度通过 sink 上报（UI 事件 / 提权进度文件 / 控制台）。

use std::path::{Path, PathBuf};
use std::time::Duration;

use serde::{Deserialize, Serialize};
use winreg::RegKey;

use super::install::{self, InstallPaths, APP_DISPLAY_NAME, MAIN_EXE_NAME, WIZARD_EXE_NAME};
use super::process;
use super::progress::{InstallProgress, ProgressSink};
use super::registry;

/// 卸载现场（自动检测或 --detected-* 透传参数重建），UI 与无头路径共用。
/// 字段名 camelCase：作为 get_install_config 的 uninstallInfo 返回给前端
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UninstallSite {
    /// true = 系统级（HKLM + Program Files），false = 用户级（HKCU + LOCALAPPDATA）
    pub is_system: bool,
    pub install_dir: String,
    /// 注册表回退检测无从得知是否创建过桌面快捷方式，保守按 true 处理
    pub created_desktop_shortcut: bool,
}

/// 自动检测安装现场。
/// 优先读取 exe 同目录的 install-info.json，回退到注册表卸载键（先 HKCU 后 HKLM）。
pub fn detect_install() -> Option<UninstallSite> {
    use winreg::enums::{HKEY_CURRENT_USER, HKEY_LOCAL_MACHINE};

    if let Ok(exe) = std::env::current_exe() {
        if let Some(dir) = exe.parent() {
            if let Ok(info) = install::load_install_info(dir) {
                return Some(UninstallSite {
                    is_system: info.is_system,
                    install_dir: info.install_dir,
                    created_desktop_shortcut: info.created_desktop_shortcut,
                });
            }
        }
    }

    let key_path = format!(
        r"Software\Microsoft\Windows\CurrentVersion\Uninstall\{}",
        registry::APP_ID
    );
    for (is_system, hive) in [(false, HKEY_CURRENT_USER), (true, HKEY_LOCAL_MACHINE)] {
        if let Ok(key) = RegKey::predef(hive).open_subkey(&key_path) {
            return Some(UninstallSite {
                is_system,
                install_dir: key.get_value("InstallLocation").unwrap_or_default(),
                created_desktop_shortcut: true,
            });
        }
    }
    None
}

/// 临时副本/提权 worker 的透传参数（`--detected-*`）：副本已不在安装目录内，
/// 无法从 exe 同目录读 install-info.json，由参数重建卸载现场。
///（`--remove-user-data` 不属于现场重建，由调用方直接从 args 解析）
#[derive(Debug, PartialEq)]
pub struct DetectedArgs {
    pub install_dir: String,
    pub is_system: bool,
    pub desktop_shortcut: bool,
}

impl DetectedArgs {
    pub fn to_site(&self) -> UninstallSite {
        UninstallSite {
            is_system: self.is_system,
            install_dir: self.install_dir.clone(),
            created_desktop_shortcut: self.desktop_shortcut,
        }
    }
}

/// 解析 `--detected-*` 透传参数。无 `--detected-install-dir` 时返回 Ok(None)
/// （走正常自动检测流程）；存在时路径须合法且必须携带
/// `--detected-system` / `--detected-user` 之一。
pub fn parse_detected_args(args: &[String]) -> Result<Option<DetectedArgs>, String> {
    let Some(install_dir) = args
        .iter()
        .find_map(|a| a.strip_prefix("--detected-install-dir="))
    else {
        return Ok(None);
    };
    super::install::validate_install_dir(install_dir)?;
    let is_system = match (
        args.iter().any(|a| a == "--detected-system"),
        args.iter().any(|a| a == "--detected-user"),
    ) {
        (true, false) => true,
        (false, true) => false,
        _ => {
            return Err(
                "--detected-install-dir 需要且只能搭配 --detected-system / --detected-user 之一"
                    .into(),
            )
        }
    };
    Ok(Some(DetectedArgs {
        install_dir: install_dir.into(),
        is_system,
        desktop_shortcut: args.iter().any(|a| a == "--detected-desktop-shortcut"),
    }))
}

/// 删除可能仍被占用的文件（临时副本卸载时原卸载器进程刚退出，
/// 文件锁可能尚未释放），轮询重试直至成功或超时。文件不存在视为成功。
pub fn remove_locked_file_with_retry(path: &Path, timeout: Duration) -> bool {
    let deadline = std::time::Instant::now() + timeout;
    loop {
        if !path.exists() || std::fs::remove_file(path).is_ok() {
            return true;
        }
        if std::time::Instant::now() >= deadline {
            return false;
        }
        std::thread::sleep(Duration::from_millis(200));
    }
}

/// 卸载所需的全部信息（由 UninstallSite 经 run_uninstall_for_site 构建）
pub struct UninstallOptions {
    pub app_id: String,
    /// 文件关联登记面参数（正式值均为常量，测试传专用值隔离）
    pub assoc: registry::AssocParams<'static>,
    pub install_dir: Option<PathBuf>,
    pub start_menu_dir: Option<PathBuf>,
    pub desktop_shortcut: Option<PathBuf>,
    /// "同时删除笔记数据"勾选时为 %APPDATA%\{app_id}
    pub user_data_dir: Option<PathBuf>,
}

/// 执行卸载，进度逐步上报，返回错误列表（空列表 = 全部成功）。
/// 完成/失败的终态事件由调用方发出（UI 侧依赖 command 返回值推进）。
pub fn run_uninstall(root: &RegKey, opts: &UninstallOptions, sink: ProgressSink) -> Vec<String> {
    let mut errors = Vec::new();

    // 1. 关闭安装目录下运行中的进程（主程序），释放 exe 文件锁
    if let Some(dir) = &opts.install_dir {
        sink(&InstallProgress::running(
            "close",
            5,
            format!("正在关闭运行中的 {APP_DISPLAY_NAME}..."),
        ));
        errors.extend(process::kill_processes_under(dir));
    }

    // 2. 删除桌面快捷方式与开始菜单目录
    sink(&InstallProgress::running(
        "shortcut",
        20,
        "正在删除快捷方式...".into(),
    ));
    if let Some(lnk) = &opts.desktop_shortcut {
        if lnk.exists() {
            if let Err(e) = std::fs::remove_file(lnk) {
                errors.push(format!("删除桌面快捷方式失败: {e}"));
            }
        }
    }
    if let Some(dir) = &opts.start_menu_dir {
        if dir.exists() {
            if let Err(e) = std::fs::remove_dir_all(dir) {
                errors.push(format!("删除开始菜单快捷方式失败: {e}"));
            }
        }
    }

    // 3. 清理文件关联登记面（幂等，未注册过也可安全调用）
    sink(&InstallProgress::running(
        "register",
        40,
        "正在清理文件关联...".into(),
    ));
    if let Err(e) = registry::unregister_file_associations(root, &opts.assoc) {
        errors.push(e);
    }

    // 4. 删除注册表卸载信息
    sink(&InstallProgress::running(
        "register",
        60,
        "正在清理注册表...".into(),
    ));
    if let Err(e) = registry::remove_uninstall_info(root, &opts.app_id) {
        errors.push(e);
    }

    // 5. 删除安装目录：主程序/原卸载器 exe 可能刚被终止或原进程刚退出，
    // 句柄释放有延迟，先带重试删除两个 exe 再整体删目录
    sink(&InstallProgress::running(
        "files",
        80,
        "正在删除程序文件...".into(),
    ));
    if let Some(dir) = &opts.install_dir {
        if dir.exists() {
            for exe in [MAIN_EXE_NAME, WIZARD_EXE_NAME] {
                let path = dir.join(exe);
                if !remove_locked_file_with_retry(&path, Duration::from_secs(10)) {
                    errors.push(format!("删除 {} 超时（文件被占用）", path.display()));
                }
            }
            if let Err(e) = std::fs::remove_dir_all(dir) {
                errors.push(format!("删除安装目录失败: {e}"));
            }
        }
    }

    // 6. （可选）删除用户数据
    if let Some(dir) = &opts.user_data_dir {
        sink(&InstallProgress::running(
            "files",
            90,
            "正在删除用户数据...".into(),
        ));
        if dir.exists() {
            if let Err(e) = std::fs::remove_dir_all(dir) {
                errors.push(format!("删除用户数据失败: {e}"));
            }
        }
    }

    errors
}

/// 按卸载现场构建标准卸载选项并执行（UI 直接卸载、提权 worker、
/// 无头 CLI 三侧共用）；错误合并为单条消息返回
pub fn run_uninstall_for_site(
    site: &UninstallSite,
    remove_user_data: bool,
    sink: ProgressSink,
) -> Result<(), String> {
    let paths = InstallPaths::resolve(site.is_system)?;
    let user_data_dir = if remove_user_data {
        Some(
            std::env::var("APPDATA")
                .map(|d| Path::new(&d).join(registry::APP_ID))
                .map_err(|_| "环境变量 APPDATA 不可用".to_string())?,
        )
    } else {
        None
    };
    let opts = UninstallOptions {
        app_id: registry::APP_ID.into(),
        assoc: registry::AssocParams {
            progid: registry::PROGID,
            extensions: registry::ASSOC_EXTENSIONS,
            exe_name: MAIN_EXE_NAME,
            capability_key: registry::CAPABILITY_KEY,
            app_name: APP_DISPLAY_NAME,
        },
        install_dir: (!site.install_dir.is_empty()).then(|| PathBuf::from(&site.install_dir)),
        start_menu_dir: Some(paths.start_menu_dir),
        desktop_shortcut: site
            .created_desktop_shortcut
            .then(|| paths.desktop_dir.join(format!("{APP_DISPLAY_NAME}.lnk"))),
        user_data_dir,
    };
    let root = registry::root_for(site.is_system);
    let errors = run_uninstall(&root, &opts, sink);
    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors.join("\n"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use winreg::enums::HKEY_CURRENT_USER;

    #[test]
    fn run_uninstall_cleans_everything() {
        let root = RegKey::predef(HKEY_CURRENT_USER);
        let app_id = "MyAppWizardTest.Uninst2";
        let assoc = registry::AssocParams {
            progid: "MyAppWizardTest.ProgId2",
            extensions: &["myapptest2"],
            exe_name: "myapp-wizard-test2.exe",
            capability_key: r"Software\MyAppWizardTest.Caps2",
            app_name: "MyAppWizardTest App2",
        };
        let ext = assoc.extensions[0];

        // 构造伪安装现场：安装目录、开始菜单目录、桌面快捷方式、注册表键、文件关联
        let base = std::env::temp_dir().join(format!(
            "my-app-uninst-test-{}",
            std::process::id()
        ));
        let install_dir = base.join("app");
        let start_menu_dir = base.join("start-menu");
        let desktop_lnk = base.join("My App.lnk");
        let user_data_dir = base.join("user-data");
        std::fs::create_dir_all(&install_dir).unwrap();
        std::fs::create_dir_all(&start_menu_dir).unwrap();
        std::fs::create_dir_all(&user_data_dir).unwrap();
        std::fs::write(install_dir.join("my-app.exe"), b"fake exe").unwrap();
        std::fs::write(start_menu_dir.join("My App.lnk"), b"fake lnk").unwrap();
        std::fs::write(&desktop_lnk, b"fake lnk").unwrap();
        std::fs::write(user_data_dir.join("notes.db"), b"fake db").unwrap();

        let info = registry::UninstallInfo {
            display_name: "My App".into(),
            display_version: "0.1.0".into(),
            publisher: "My Company".into(),
            uninstall_string: format!("\"{}\" --uninstall", install_dir.display()),
            install_location: install_dir.to_string_lossy().into_owned(),
            display_icon: install_dir.join("my-app.exe").to_string_lossy().into_owned(),
            estimated_size_kb: 1,
        };
        registry::write_uninstall_info(&root, app_id, &info).unwrap();
        registry::register_file_associations(
            &root,
            &assoc,
            &install_dir.join("my-app.exe").to_string_lossy(),
        )
        .unwrap();

        let opts = UninstallOptions {
            app_id: app_id.into(),
            assoc,
            install_dir: Some(install_dir.clone()),
            start_menu_dir: Some(start_menu_dir.clone()),
            desktop_shortcut: Some(desktop_lnk.clone()),
            user_data_dir: Some(user_data_dir.clone()),
        };
        let progress_log = std::cell::RefCell::new(Vec::new());
        let errors = run_uninstall(&root, &opts, &|p| {
            progress_log.borrow_mut().push(p.clone());
        });
        assert!(errors.is_empty(), "卸载出现错误: {errors:?}");

        // 全部清理干净（含主程序 exe 与用户数据）
        assert!(!install_dir.exists());
        assert!(!start_menu_dir.exists());
        assert!(!desktop_lnk.exists());
        assert!(!user_data_dir.exists());
        assert!(root
            .open_subkey(format!(
                r"Software\Microsoft\Windows\CurrentVersion\Uninstall\{app_id}"
            ))
            .is_err());
        assert!(root
            .open_subkey(format!(r"Software\Classes\{}", assoc.progid))
            .is_err());
        // 登记面同步清理：Applications/Capabilities/RegisteredApplications 登记消失
        assert!(root
            .open_subkey(format!(r"Software\Classes\Applications\{}", assoc.exe_name))
            .is_err());
        assert!(root.open_subkey(assoc.capability_key).is_err());
        let reg_apps = root.open_subkey(r"Software\RegisteredApplications").unwrap();
        assert!(reg_apps.get_value::<String, _>(assoc.app_name).is_err());
        drop(reg_apps);

        // 进度：逐步上报且百分比递增，含关闭主程序与删除用户数据步骤
        let log = progress_log.borrow();
        let steps: Vec<&str> = log.iter().map(|p| p.step.as_str()).collect();
        assert_eq!(steps, vec!["close", "shortcut", "register", "register", "files", "files"]);
        assert!(log.windows(2).all(|w| w[0].percent <= w[1].percent));

        // 清理测试残留（扩展名键、临时目录）
        let _ = root.delete_subkey_all(format!(r"Software\Classes\.{ext}"));
        let _ = std::fs::remove_dir_all(&base);
    }

    #[test]
    fn run_uninstall_is_tolerant_when_nothing_exists() {
        let root = RegKey::predef(HKEY_CURRENT_USER);
        let base = std::env::temp_dir().join("my-app-uninst-test-missing");

        let opts = UninstallOptions {
            app_id: "MyAppWizardTest.NotExist".into(),
            assoc: registry::AssocParams {
                progid: "MyAppWizardTest.NotExist",
                extensions: &["myapptestmissing"],
                exe_name: "myapp-wizard-test-missing.exe",
                capability_key: r"Software\MyAppWizardTest.CapsMissing",
                app_name: "MyAppWizardTest AppMissing",
            },
            install_dir: Some(base.join("app")),
            start_menu_dir: Some(base.join("start-menu")),
            desktop_shortcut: Some(base.join("My App.lnk")),
            user_data_dir: None,
        };
        // 无安装现场时不应产生任何错误
        let errors = run_uninstall(&root, &opts, &|_| {});
        assert!(errors.is_empty(), "空现场卸载不应报错: {errors:?}");
    }

    fn to_args(args: &[&str]) -> Vec<String> {
        args.iter().map(|s| s.to_string()).collect()
    }

    #[test]
    fn parse_detected_args_absent_is_none() {
        assert_eq!(
            parse_detected_args(&to_args(&["--uninstall", "--confirm"])).unwrap(),
            None
        );
    }

    #[test]
    fn parse_detected_args_full_flags() {
        let parsed = parse_detected_args(&to_args(&[
            "--uninstall",
            "--confirm",
            r"--detected-install-dir=C:\Fake\My App",
            "--detected-system",
            "--detected-desktop-shortcut",
        ]))
        .unwrap();
        assert_eq!(
            parsed,
            Some(DetectedArgs {
                install_dir: r"C:\Fake\My App".into(),
                is_system: true,
                desktop_shortcut: true,
            })
        );
    }

    #[test]
    fn parse_detected_args_user_level_defaults() {
        let parsed = parse_detected_args(&to_args(&[
            r"--detected-install-dir=C:\Fake\My App",
            "--detected-user",
        ]))
        .unwrap()
        .unwrap();
        assert!(!parsed.is_system);
        assert!(!parsed.desktop_shortcut);
        assert_eq!(parsed.to_site().install_dir, r"C:\Fake\My App");
    }

    #[test]
    fn parse_detected_args_rejects_invalid_input() {
        // 缺少安装类型标记
        assert!(
            parse_detected_args(&to_args(&[r"--detected-install-dir=C:\Fake\My App"])).is_err()
        );
        // 两个标记同时存在
        assert!(parse_detected_args(&to_args(&[
            r"--detected-install-dir=C:\Fake\My App",
            "--detected-system",
            "--detected-user",
        ]))
        .is_err());
        // 相对路径
        assert!(parse_detected_args(&to_args(&[
            r"--detected-install-dir=relative\dir",
            "--detected-user",
        ]))
        .is_err());
    }

    #[test]
    fn remove_locked_file_with_retry_handles_missing_and_existing() {
        let base = std::env::temp_dir().join(format!(
            "my-app-remove-retry-test-{}",
            std::process::id()
        ));
        std::fs::create_dir_all(&base).unwrap();

        // 文件不存在视为成功
        assert!(remove_locked_file_with_retry(
            &base.join("missing.exe"),
            Duration::from_millis(100)
        ));

        // 未被占用的文件直接删除
        let file = base.join("wizard.exe");
        std::fs::write(&file, b"fake exe").unwrap();
        assert!(remove_locked_file_with_retry(
            &file,
            Duration::from_millis(100)
        ));
        assert!(!file.exists());

        let _ = std::fs::remove_dir_all(&base);
    }
}
