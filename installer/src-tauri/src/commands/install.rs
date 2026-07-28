// Copyright (c) 2026 青山公仔
// 轻壳 (Tauri Shell) is licensed under Mulan PSL v2.
// You can use this software according to the terms and conditions of the Mulan PSL v2.
// You may obtain a copy of Mulan PSL v2 at:
//          http://license.coscl.org.cn/MulanPSL2
// THIS SOFTWARE IS PROVIDED ON AN "AS IS" BASIS, WITHOUT WARRANTIES OF ANY KIND,
// EITHER EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO NON-INFRINGEMENT,
// MERCHANTABILITY OR FIT FOR A PARTICULAR PURPOSE.
// See the Mulan PSL v2 for more details.

//! 安装编排：嵌入包解压、卸载器复制、元信息/注册表/快捷方式写入，
//! 及安装信息持久化与双路径解析。
//!
//! 直接安装与 `--elevated` 提权安装共用 `run_install` 核心，仅进度 sink 不同
//! （Tauri 事件 vs 进度文件）。

use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use winreg::RegKey;

use super::progress::{self, InstallProgress, ProgressSink};
use super::{extract, process, registry, shortcut};

/// 安装目录下记录安装元信息的文件名
pub const INSTALL_INFO_FILE: &str = "install-info.json";
// 产品常量集中在 config.rs，此处再导出以保持既有引用路径
pub use crate::config::{APP_DISPLAY_NAME, MAIN_EXE_NAME, PUBLISHER, WIZARD_EXE_NAME};

/// 安装时写入安装目录的元信息，卸载时读取以还原当时的安装类型选择。
/// 字段名 camelCase：与第四阶段前端向导页的 IPC 字段契约保持一致
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InstallInfo {
    /// true = 系统级（HKLM + Program Files），false = 用户级（HKCU + LOCALAPPDATA）
    pub is_system: bool,
    pub version: String,
    pub install_dir: String,
    pub app_id: String,
    pub progid: String,
    /// 安装时是否创建了桌面快捷方式，卸载时据此决定是否删除
    pub created_desktop_shortcut: bool,
    /// 安装时是否登记了文件关联，更新模式据此静默沿用上次选择
    pub file_assoc: bool,
}

/// 将安装信息写入 `{dir}\install-info.json`
pub fn save_install_info(dir: &Path, info: &InstallInfo) -> Result<(), String> {
    let json = serde_json::to_string_pretty(info)
        .map_err(|e| format!("序列化安装信息失败: {e}"))?;
    std::fs::create_dir_all(dir).map_err(|e| format!("创建安装目录失败: {e}"))?;
    std::fs::write(dir.join(INSTALL_INFO_FILE), json)
        .map_err(|e| format!("写入 {INSTALL_INFO_FILE} 失败: {e}"))
}

/// 从 `{dir}\install-info.json` 读取安装信息；文件缺失或损坏返回 Err
pub fn load_install_info(dir: &Path) -> Result<InstallInfo, String> {
    let path = dir.join(INSTALL_INFO_FILE);
    let json = std::fs::read_to_string(&path)
        .map_err(|e| format!("读取 {} 失败: {e}", path.display()))?;
    serde_json::from_str(&json).map_err(|e| format!("解析 {} 失败: {e}", path.display()))
}

/// 系统级/用户级安装对应的标准路径集合
pub struct InstallPaths {
    /// 默认安装目录（用户可在向导中修改）
    pub default_install_dir: PathBuf,
    /// 开始菜单程序组目录
    pub start_menu_dir: PathBuf,
    /// 桌面目录（快捷方式放置于此）
    pub desktop_dir: PathBuf,
}

fn env_dir(name: &str) -> Result<PathBuf, String> {
    std::env::var(name)
        .map(PathBuf::from)
        .map_err(|_| format!("环境变量 {name} 不可用"))
}

impl InstallPaths {
    /// 根据安装类型解析标准路径（见方案文档"安装路径与对应权限"表）
    pub fn resolve(is_system: bool) -> Result<Self, String> {
        let dir_name = crate::config::INSTALL_DIR_NAME;
        let start_menu_suffix =
            Path::new(r"Microsoft\Windows\Start Menu\Programs").join(dir_name);
        if is_system {
            Ok(Self {
                default_install_dir: env_dir("ProgramFiles")?.join(dir_name),
                start_menu_dir: env_dir("ProgramData")?.join(&start_menu_suffix),
                desktop_dir: env_dir("PUBLIC")?.join("Desktop"),
            })
        } else {
            Ok(Self {
                default_install_dir: env_dir("LOCALAPPDATA")?.join("Programs").join(dir_name),
                start_menu_dir: env_dir("APPDATA")?.join(&start_menu_suffix),
                desktop_dir: env_dir("USERPROFILE")?.join("Desktop"),
            })
        }
    }
}

/// 前端 `start_install` 参数（camelCase IPC 契约）；提权路径由 CLI 参数重建
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InstallOptions {
    /// true = 系统级（HKLM + Program Files），false = 用户级（HKCU + LOCALAPPDATA）
    pub is_system: bool,
    pub install_dir: String,
    pub desktop_shortcut: bool,
    pub file_assoc: bool,
}

/// 安装编排的环境依赖，正式安装由 `run_install` 构造；
/// 测试注入隔离现场（HKCU 专用键 + %TEMP% 目录 + 测试 zip）
pub struct InstallContext<'a> {
    pub root: RegKey,
    pub app_id: &'a str,
    pub assoc: registry::AssocParams<'a>,
    pub start_menu_dir: PathBuf,
    pub desktop_dir: PathBuf,
    pub bundle: &'a [u8],
    pub version: &'a str,
}

/// 校验安装目录：非空、绝对路径、不含引号（引号会破坏提权命令行引号结构，
/// 且 Windows 路径本就不允许）。直接安装与提权安装两侧入口统一调用。
pub fn validate_install_dir(dir: &str) -> Result<(), String> {
    if dir.trim().is_empty() {
        return Err("安装路径不能为空".into());
    }
    if dir.contains('"') {
        return Err("安装路径不能包含引号".into());
    }
    if !Path::new(dir).is_absolute() {
        return Err(format!("安装路径必须为绝对路径: {dir}"));
    }
    Ok(())
}

/// 执行安装（正式环境：嵌入包 + 标准路径 + 正式注册表键）
pub fn run_install(opts: &InstallOptions, sink: ProgressSink) -> Result<(), String> {
    let bundle = extract::embedded_bundle()?;
    let paths = InstallPaths::resolve(opts.is_system)?;
    let ctx = InstallContext {
        root: registry::root_for(opts.is_system),
        app_id: registry::APP_ID,
        assoc: registry::AssocParams {
            progid: registry::PROGID,
            extensions: registry::ASSOC_EXTENSIONS,
            exe_name: MAIN_EXE_NAME,
            capability_key: registry::CAPABILITY_KEY,
            app_name: APP_DISPLAY_NAME,
        },
        start_menu_dir: paths.start_menu_dir,
        desktop_dir: paths.desktop_dir,
        bundle,
        version: env!("CARGO_PKG_VERSION"),
    };
    run_install_with(opts, &ctx, sink)
}

/// 安装编排核心，每步上报进度：关闭运行中进程（覆盖安装）→
/// 解压（0-80%）→ 卸载器就位 → 元信息 → 注册表 → 文件关联（可选）→
/// 快捷方式（COM 失败仅记警告不中断）
pub fn run_install_with(
    opts: &InstallOptions,
    ctx: &InstallContext,
    sink: ProgressSink,
) -> Result<(), String> {
    let install_dir = PathBuf::from(&opts.install_dir);

    // 0. 覆盖安装：目标目录已存在时先关闭其中运行的进程（主程序/旧卸载器），
    // 否则 exe 文件锁会导致解压覆盖失败；终止失败仅记警告，
    // 真实占用会在解压时以 IO 错误暴露
    if install_dir.exists() {
        sink(&InstallProgress::running(
            "close",
            0,
            format!("正在关闭运行中的 {APP_DISPLAY_NAME}..."),
        ));
        for e in process::kill_processes_under(&install_dir) {
            sink(&InstallProgress::running("close", 0, format!("警告: {e}")));
        }
    }

    // 1. 解压嵌入包；按 percent 去重上报，避免条目多时事件洪泛与
    // 提权进度文件的 IO 放大
    sink(&InstallProgress::running(
        "extract",
        0,
        "正在解压程序文件...".into(),
    ));
    let last_percent = std::cell::Cell::new(0u8);
    let extracted_bytes = extract::extract_zip(ctx.bundle, &install_dir, &|done, total| {
        let percent = (done * 80 / total.max(1)) as u8;
        if last_percent.replace(percent) != percent {
            sink(&InstallProgress::running(
                "extract",
                percent,
                format!("正在解压程序文件 ({done}/{total})..."),
            ));
        }
    })?;

    // 2. 卸载器：由 zip 内的纯壳解压就位（第六阶段双产物流程，
    // 避免卸载器内嵌无用 zip 的体积浪费），缺壳视为打包错误中断安装
    sink(&InstallProgress::running(
        "copy",
        82,
        "正在复制卸载程序...".into(),
    ));
    let wizard_dest = install_dir.join(WIZARD_EXE_NAME);
    if !wizard_dest.exists() {
        return Err(format!(
            "安装包未含卸载程序 {WIZARD_EXE_NAME}，安装包损坏或打包错误"
        ));
    }

    // 3. 安装元信息（卸载时自动检测安装类型的依据）
    save_install_info(
        &install_dir,
        &InstallInfo {
            is_system: opts.is_system,
            version: ctx.version.into(),
            install_dir: opts.install_dir.clone(),
            app_id: ctx.app_id.into(),
            progid: ctx.assoc.progid.into(),
            created_desktop_shortcut: opts.desktop_shortcut,
            file_assoc: opts.file_assoc,
        },
    )?;

    // 4. 控制面板卸载信息
    sink(&InstallProgress::running(
        "register",
        88,
        "正在写入注册表...".into(),
    ));
    let main_exe = install_dir.join(MAIN_EXE_NAME);
    registry::write_uninstall_info(
        &ctx.root,
        ctx.app_id,
        &registry::UninstallInfo {
            display_name: APP_DISPLAY_NAME.into(),
            display_version: ctx.version.into(),
            publisher: PUBLISHER.into(),
            // 控制面板"卸载"进卸载向导 UI（UI 确认页即安全闸）
            uninstall_string: format!("\"{}\" --uninstall", wizard_dest.display()),
            install_location: opts.install_dir.clone(),
            display_icon: main_exe.to_string_lossy().into_owned(),
            estimated_size_kb: (extracted_bytes / 1024).min(u32::MAX as u64) as u32,
        },
    )?;

    // 5. 文件关联（用户可选；config.rs 未配置扩展名时功能整体关闭）
    if opts.file_assoc && !ctx.assoc.extensions.is_empty() {
        sink(&InstallProgress::running(
            "register",
            92,
            "正在注册文件关联...".into(),
        ));
        registry::register_file_associations(&ctx.root, &ctx.assoc, &main_exe.to_string_lossy())?;
    }

    // 6. 快捷方式
    sink(&InstallProgress::running(
        "shortcut",
        96,
        "正在创建快捷方式...".into(),
    ));
    create_app_shortcuts(opts, ctx, &main_exe, &install_dir, sink);

    sink(&InstallProgress::finished());
    Ok(())
}

/// 创建开始菜单（始终）与桌面（按选项）快捷方式
fn create_app_shortcuts(
    opts: &InstallOptions,
    ctx: &InstallContext,
    main_exe: &Path,
    install_dir: &Path,
    sink: ProgressSink,
) {
    let main_exe_str = main_exe.to_string_lossy();
    let install_dir_str = install_dir.to_string_lossy();
    let lnk_name = format!("{APP_DISPLAY_NAME}.lnk");
    let mut shortcut_targets = vec![ctx.start_menu_dir.join(&lnk_name)];
    if opts.desktop_shortcut {
        shortcut_targets.push(ctx.desktop_dir.join(&lnk_name));
    }
    for lnk in &shortcut_targets {
        if let Err(e) = shortcut::create_shortcut(
            &main_exe_str,
            &lnk.to_string_lossy(),
            APP_DISPLAY_NAME,
            &install_dir_str,
            &main_exe_str,
        ) {
            // COM 失败仅记警告不中断安装（方案文档既定对策）
            sink(&InstallProgress::running(
                "shortcut",
                97,
                format!("警告: 创建快捷方式 {} 失败（{e}），已跳过", lnk.display()),
            ));
        }
    }
}

/// 检测到的已安装现场（更新模式的数据源）。
/// 字段名 camelCase：作为 get_install_config 的 updateInfo 返回给前端
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ExistingInstall {
    /// true = 系统级（HKLM + Program Files），false = 用户级（HKCU + LOCALAPPDATA）
    pub is_system: bool,
    pub install_dir: String,
    /// 已安装的版本号（取自 install-info.json）
    pub version: String,
    pub desktop_shortcut: bool,
    pub file_assoc: bool,
}

/// 检测已有安装（安装器无参数启动时决定进入更新/新安装模式）。
/// 查注册表卸载键（先 HKCU 后 HKLM，与卸载侧检测顺序一致）取 InstallLocation，
/// 再从该目录读 install-info.json 补全上次的安装选项。
/// 与卸载侧 detect_install 不合并：后者优先读 exe 同目录的元信息（卸载器在
/// 安装目录内），安装器从下载目录运行，只能走"注册表键 → 安装目录 json"这条链
pub fn detect_existing_install() -> Option<ExistingInstall> {
    use winreg::enums::{HKEY_CURRENT_USER, HKEY_LOCAL_MACHINE};

    let key_path = format!(
        r"Software\Microsoft\Windows\CurrentVersion\Uninstall\{}",
        registry::APP_ID
    );
    for hive in [HKEY_CURRENT_USER, HKEY_LOCAL_MACHINE] {
        if let Ok(key) = RegKey::predef(hive).open_subkey(&key_path) {
            if let Some(found) = existing_install_from_key(&key) {
                return Some(found);
            }
        }
    }
    None
}

/// 从卸载键的 InstallLocation 读 install-info.json 重建已安装现场；
/// 任一环节缺失/损坏返回 None（现场残缺回退新安装模式，天然自愈）。
/// is_system 以 json 为准（不以命中的注册表 hive 为准），与卸载检测同源。
/// 拆为独立函数供测试用 HKCU 专用键验证。
fn existing_install_from_key(key: &RegKey) -> Option<ExistingInstall> {
    let install_dir: String = key.get_value("InstallLocation").ok()?;
    if install_dir.is_empty() {
        return None;
    }
    let info = load_install_info(Path::new(&install_dir)).ok()?;
    Some(ExistingInstall {
        is_system: info.is_system,
        install_dir,
        version: info.version,
        desktop_shortcut: info.created_desktop_shortcut,
        file_assoc: info.file_assoc,
    })
}

/// `--elevated` 模式的 CLI 参数（UI 进程构造，提权进程重建 InstallOptions）
#[derive(Debug, PartialEq)]
pub struct ElevatedArgs {
    pub install_path: String,
    pub desktop_shortcut: bool,
    pub file_assoc: bool,
    pub progress_file: Option<PathBuf>,
}

/// 解析 `--elevated` 模式参数：`--install-path=` 必填且须通过安装目录校验
pub fn parse_elevated_args(args: &[String]) -> Result<ElevatedArgs, String> {
    let install_path = args
        .iter()
        .find_map(|a| a.strip_prefix("--install-path="))
        .ok_or("缺少 --install-path 参数")?;
    validate_install_dir(install_path)?;
    Ok(ElevatedArgs {
        install_path: install_path.into(),
        desktop_shortcut: args.iter().any(|a| a == "--desktop-shortcut"),
        file_assoc: args.iter().any(|a| a == "--file-assoc"),
        progress_file: args
            .iter()
            .find_map(|a| a.strip_prefix("--progress-file="))
            .map(PathBuf::from),
    })
}

/// `--elevated` 模式入口：解析参数后以进度文件 sink 执行系统级安装；
/// 失败时额外将错误写入进度文件，供 UI 进程轮询呈现
pub fn run_elevated_install(args: &[String]) -> Result<(), String> {
    let parsed = parse_elevated_args(args)?;
    // 未提权时提前报错，避免解压到 Program Files 半途权限拒绝留下半成品
    if !super::elevate::is_elevated() {
        return Err("--elevated 模式需要管理员权限（应由安装向导的 UAC 提权流程启动）".into());
    }
    let progress_file = parsed.progress_file.clone();
    let sink = move |p: &InstallProgress| {
        if let Some(path) = &progress_file {
            let _ = progress::write_progress_file(path, p);
        } else {
            // 无进度文件时（提权终端手动运行）退化为控制台输出
            println!("[{:3}%] {}", p.percent, p.message);
        }
    };
    let opts = InstallOptions {
        is_system: true,
        install_dir: parsed.install_path.clone(),
        desktop_shortcut: parsed.desktop_shortcut,
        file_assoc: parsed.file_assoc,
    };
    let result = run_install(&opts, &sink);
    if let Err(e) = &result {
        if let Some(path) = &parsed.progress_file {
            let _ = progress::write_progress_file(path, &InstallProgress::failed(e.clone()));
        }
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_info(dir: &Path) -> InstallInfo {
        InstallInfo {
            is_system: true,
            version: "0.1.0".into(),
            install_dir: dir.to_string_lossy().into_owned(),
            app_id: crate::config::APP_ID.into(),
            progid: crate::config::PROGID.into(),
            created_desktop_shortcut: true,
            file_assoc: true,
        }
    }

    #[test]
    fn install_info_roundtrip() {
        let dir = std::env::temp_dir().join(format!(
            "my-app-install-info-test-{}",
            std::process::id()
        ));
        let info = sample_info(&dir);

        save_install_info(&dir, &info).unwrap();
        let loaded = load_install_info(&dir).unwrap();
        assert_eq!(loaded, info);

        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn load_install_info_missing_file_is_err() {
        let dir = std::env::temp_dir().join("my-app-install-info-missing");
        assert!(load_install_info(&dir).is_err());
    }

    #[test]
    fn load_install_info_corrupt_json_is_err() {
        let dir = std::env::temp_dir().join(format!(
            "my-app-install-info-corrupt-{}",
            std::process::id()
        ));
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join(INSTALL_INFO_FILE), "not json{{").unwrap();

        assert!(load_install_info(&dir).is_err());

        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn resolve_user_level_paths() {
        let paths = InstallPaths::resolve(false).unwrap();
        let appdata = std::env::var("APPDATA").unwrap();
        let userprofile = std::env::var("USERPROFILE").unwrap();
        let localappdata = std::env::var("LOCALAPPDATA").unwrap();

        assert!(paths.default_install_dir.starts_with(&localappdata));
        assert!(paths.start_menu_dir.starts_with(&appdata));
        assert!(paths.desktop_dir.starts_with(&userprofile));
        assert!(paths.start_menu_dir.ends_with(crate::config::INSTALL_DIR_NAME));
    }

    #[test]
    fn resolve_system_level_paths() {
        let paths = InstallPaths::resolve(true).unwrap();
        let program_files = std::env::var("ProgramFiles").unwrap();
        let program_data = std::env::var("ProgramData").unwrap();
        let public = std::env::var("PUBLIC").unwrap();

        assert!(paths.default_install_dir.starts_with(&program_files));
        assert!(paths.start_menu_dir.starts_with(&program_data));
        assert!(paths.desktop_dir.starts_with(&public));
    }

    #[test]
    fn validate_install_dir_rules() {
        assert!(validate_install_dir("").is_err());
        assert!(validate_install_dir("   ").is_err());
        assert!(validate_install_dir(r"relative\dir").is_err());
        assert!(validate_install_dir("C:\\Fake\\\"My App").is_err());
        assert!(validate_install_dir(r"C:\Fake\My App").is_ok());
    }

    #[test]
    fn parse_elevated_args_requires_install_path() {
        assert!(parse_elevated_args(&["--elevated".into()]).is_err());
        assert!(parse_elevated_args(&["--elevated".into(), "--install-path=".into()]).is_err());
        assert!(parse_elevated_args(&[
            "--elevated".into(),
            "--install-path=relative\\dir".into()
        ])
        .is_err());
    }

    #[test]
    fn parse_elevated_args_full_flags() {
        let parsed = parse_elevated_args(&[
            "--elevated".into(),
            r"--install-path=C:\Fake\My App".into(),
            "--desktop-shortcut".into(),
            "--file-assoc".into(),
            r"--progress-file=C:\Temp\p.json".into(),
        ])
        .unwrap();
        assert_eq!(
            parsed,
            ElevatedArgs {
                install_path: r"C:\Fake\My App".into(),
                desktop_shortcut: true,
                file_assoc: true,
                progress_file: Some(PathBuf::from(r"C:\Temp\p.json")),
            }
        );
    }

    #[test]
    fn parse_elevated_args_defaults_are_off() {
        let parsed = parse_elevated_args(&[
            "--elevated".into(),
            r"--install-path=C:\Fake\My App".into(),
        ])
        .unwrap();
        assert!(!parsed.desktop_shortcut);
        assert!(!parsed.file_assoc);
        assert_eq!(parsed.progress_file, None);
    }

    #[test]
    fn existing_install_from_key_rebuilds_site() {
        // Drop 守卫：断言失败 panic 时也能清理 HKCU 测试键与临时目录
        struct DetectSiteCleanup {
            key_path: &'static str,
            dir: PathBuf,
        }
        impl Drop for DetectSiteCleanup {
            fn drop(&mut self) {
                let _ = registry::root_for(false).delete_subkey_all(self.key_path);
                let _ = std::fs::remove_dir_all(&self.dir);
            }
        }

        const KEY_PATH: &str = r"Software\MyAppWizardTest.DetectSite";
        let dir = std::env::temp_dir().join(format!(
            "my-app-detect-site-test-{}",
            std::process::id()
        ));
        let _cleanup = DetectSiteCleanup {
            key_path: KEY_PATH,
            dir: dir.clone(),
        };

        let (key, _) = registry::root_for(false).create_subkey(KEY_PATH).unwrap();
        key.set_value("InstallLocation", &dir.to_string_lossy().into_owned())
            .unwrap();

        // 1. 完整现场：InstallLocation + 目录内 install-info.json → 检测成功且字段正确；
        // 测试键在 HKCU 而 json 记 is_system=true，验证 is_system 以 json 为准
        let info = sample_info(&dir);
        save_install_info(&dir, &info).unwrap();
        assert_eq!(
            existing_install_from_key(&key).unwrap(),
            ExistingInstall {
                is_system: true,
                install_dir: dir.to_string_lossy().into_owned(),
                version: info.version.clone(),
                desktop_shortcut: info.created_desktop_shortcut,
                file_assoc: info.file_assoc,
            }
        );

        // 2. json 损坏 → None；json 删除 → None
        std::fs::write(dir.join(INSTALL_INFO_FILE), "not json{{").unwrap();
        assert_eq!(existing_install_from_key(&key), None);
        std::fs::remove_file(dir.join(INSTALL_INFO_FILE)).unwrap();
        assert_eq!(existing_install_from_key(&key), None);

        // 3. InstallLocation 置空 → None；删除该值 → None
        key.set_value("InstallLocation", &"").unwrap();
        assert_eq!(existing_install_from_key(&key), None);
        key.delete_value("InstallLocation").unwrap();
        assert_eq!(existing_install_from_key(&key), None);
        // 清理由 DetectSiteCleanup 的 Drop 执行
    }

    /// 测试现场清理守卫：Drop 中执行，断言失败 panic 时也能清理，
    /// 避免残留的卸载键出现在 Windows 设置的"安装的应用"列表中
    struct SiteCleanup {
        app_id: &'static str,
        assoc: registry::AssocParams<'static>,
        base: PathBuf,
    }
    impl SiteCleanup {
        fn clean(&self) {
            let root = registry::root_for(false);
            let _ = root.delete_subkey_all(format!(
                r"Software\Microsoft\Windows\CurrentVersion\Uninstall\{}",
                self.app_id
            ));
            // 登记面清理复用反注册的幂等性
            let _ = registry::unregister_file_associations(&root, &self.assoc);
            for ext in self.assoc.extensions {
                let _ = root.delete_subkey_all(format!(r"Software\Classes\.{ext}"));
            }
            let _ = std::fs::remove_dir_all(&self.base);
        }
    }
    impl Drop for SiteCleanup {
        fn drop(&mut self) {
            self.clean();
        }
    }

    #[test]
    fn run_install_with_full_flow_in_isolated_environment() {
        use crate::commands::extract::test_util::build_test_zip;

        // 隔离现场：%TEMP% 目录 + HKCU 测试专用键，不触真实 .md 关联
        let base = std::env::temp_dir().join(format!(
            "my-app-run-install-test-{}",
            std::process::id()
        ));
        let install_dir = base.join("app");
        let app_id = "MyAppWizardTest.Install3";
        let assoc = registry::AssocParams {
            progid: "MyAppWizardTest.ProgId5",
            extensions: &["myapptest5"],
            exe_name: "myapp-wizard-test5.exe",
            capability_key: r"Software\MyAppWizardTest.Caps5",
            app_name: "MyAppWizardTest App5",
        };
        let cleanup = SiteCleanup {
            app_id,
            assoc,
            base: base.clone(),
        };
        // 先自愈：注册表 id 固定，上次进程被中断（如管道截断）遗留的键在此清掉
        cleanup.clean();

        let bundle = build_test_zip(&[
            (MAIN_EXE_NAME, b"fake main exe".as_slice()),
            (WIZARD_EXE_NAME, b"fake shell wizard".as_slice()),
            ("resources/app.json", b"{}".as_slice()),
        ]);

        let root = registry::root_for(false);
        let opts = InstallOptions {
            is_system: false,
            install_dir: install_dir.to_string_lossy().into_owned(),
            desktop_shortcut: true,
            file_assoc: true,
        };
        let ctx = InstallContext {
            root: registry::root_for(false),
            app_id,
            assoc,
            start_menu_dir: base.join("start-menu"),
            desktop_dir: base.join("desktop"),
            bundle: &bundle,
            version: "9.9.9",
        };

        let progress_log = std::cell::RefCell::new(Vec::new());
        run_install_with(&opts, &ctx, &|p| {
            progress_log.borrow_mut().push(p.clone());
        })
        .unwrap();

        // 文件：主程序与卸载器均自 zip 解压、元信息写入
        assert_eq!(
            std::fs::read(install_dir.join(MAIN_EXE_NAME)).unwrap(),
            b"fake main exe"
        );
        assert_eq!(
            std::fs::read(install_dir.join(WIZARD_EXE_NAME)).unwrap(),
            b"fake shell wizard"
        );
        let info = load_install_info(&install_dir).unwrap();
        assert!(!info.is_system);
        assert_eq!(info.version, "9.9.9");
        assert!(info.created_desktop_shortcut);
        assert!(info.file_assoc);

        // 注册表：卸载信息与文件关联
        let key = root
            .open_subkey(format!(
                r"Software\Microsoft\Windows\CurrentVersion\Uninstall\{app_id}"
            ))
            .unwrap();
        assert_eq!(
            key.get_value::<String, _>("DisplayName").unwrap(),
            APP_DISPLAY_NAME
        );
        let uninstall_string: String = key.get_value("UninstallString").unwrap();
        assert!(uninstall_string.ends_with("--uninstall"));
        assert!(!uninstall_string.contains("--confirm"));
        drop(key);
        // 登记面：OpenWithProgids 挂上候选且不写扩展名默认值，RegisteredApplications 登记存在
        let ext = assoc.extensions[0];
        let progids_key = root
            .open_subkey(format!(r"Software\Classes\.{ext}\OpenWithProgids"))
            .unwrap();
        assert_eq!(
            progids_key
                .get_value::<String, _>(assoc.progid)
                .unwrap(),
            ""
        );
        drop(progids_key);
        let ext_key = root
            .open_subkey(format!(r"Software\Classes\.{ext}"))
            .unwrap();
        assert_eq!(ext_key.get_value::<String, _>("").unwrap_or_default(), "");
        drop(ext_key);
        let reg_apps = root.open_subkey(r"Software\RegisteredApplications").unwrap();
        assert_eq!(
            reg_apps.get_value::<String, _>(assoc.app_name).unwrap(),
            format!(r"{}\Capabilities", assoc.capability_key)
        );
        drop(reg_apps);

        // 快捷方式：开始菜单 + 桌面（测试目录）
        let lnk_name = format!("{APP_DISPLAY_NAME}.lnk");
        assert!(base.join("start-menu").join(&lnk_name).exists());
        assert!(base.join("desktop").join(&lnk_name).exists());

        // 进度：首条为 extract，末条 done 且 100%
        let log = progress_log.borrow();
        assert_eq!(log.first().unwrap().step, "extract");
        let last = log.last().unwrap();
        assert!(last.done && last.percent == 100 && last.error.is_none());
        // 清理由 SiteCleanup 的 Drop 执行
    }

    #[test]
    fn run_install_with_fails_when_bundle_missing_wizard() {
        use crate::commands::extract::test_util::build_test_zip;

        let base = std::env::temp_dir().join(format!(
            "my-app-missing-wizard-test-{}",
            std::process::id()
        ));
        let install_dir = base.join("app");
        let app_id = "MyAppWizardTest.Install6";
        let assoc = registry::AssocParams {
            progid: "MyAppWizardTest.ProgId6",
            extensions: &["myapptest6"],
            exe_name: "myapp-wizard-test6.exe",
            capability_key: r"Software\MyAppWizardTest.Caps6",
            app_name: "MyAppWizardTest App6",
        };
        let cleanup = SiteCleanup {
            app_id,
            assoc,
            base: base.clone(),
        };
        cleanup.clean();

        // 缺壳 zip 属打包错误，安装应报错中断而非回退复制自身
        let bundle = build_test_zip(&[(MAIN_EXE_NAME, b"fake main exe".as_slice())]);

        let opts = InstallOptions {
            is_system: false,
            install_dir: install_dir.to_string_lossy().into_owned(),
            desktop_shortcut: false,
            file_assoc: false,
        };
        let ctx = InstallContext {
            root: registry::root_for(false),
            app_id,
            assoc,
            start_menu_dir: base.join("start-menu"),
            desktop_dir: base.join("desktop"),
            bundle: &bundle,
            version: "9.9.9",
        };

        let err = run_install_with(&opts, &ctx, &|_| {}).unwrap_err();
        assert!(err.contains("未含卸载程序"));
        assert!(!install_dir.join(WIZARD_EXE_NAME).exists());
        // 错误发生在注册表写入之前，不应留下卸载键
        assert!(ctx
            .root
            .open_subkey(format!(
                r"Software\Microsoft\Windows\CurrentVersion\Uninstall\{app_id}"
            ))
            .is_err());
        // 清理由 SiteCleanup 的 Drop 执行
    }

    #[test]
    fn run_install_with_closes_running_processes_on_overwrite() {
        use crate::commands::extract::test_util::build_test_zip;

        let base = std::env::temp_dir().join(format!(
            "my-app-overwrite-test-{}",
            std::process::id()
        ));
        let install_dir = base.join("app");
        let app_id = "MyAppWizardTest.Install7";
        let assoc = registry::AssocParams {
            progid: "MyAppWizardTest.ProgId7",
            extensions: &["myapptest7"],
            exe_name: "myapp-wizard-test7.exe",
            capability_key: r"Software\MyAppWizardTest.Caps7",
            app_name: "MyAppWizardTest App7",
        };
        let cleanup = SiteCleanup {
            app_id,
            assoc,
            base: base.clone(),
        };
        cleanup.clean();

        // 覆盖安装现场：目标目录已存在（含旧主程序，无运行中进程）
        std::fs::create_dir_all(&install_dir).unwrap();
        std::fs::write(install_dir.join(MAIN_EXE_NAME), b"old main exe").unwrap();

        let bundle = build_test_zip(&[
            (MAIN_EXE_NAME, b"new main exe".as_slice()),
            (WIZARD_EXE_NAME, b"fake shell wizard".as_slice()),
        ]);

        let opts = InstallOptions {
            is_system: false,
            install_dir: install_dir.to_string_lossy().into_owned(),
            desktop_shortcut: false,
            file_assoc: false,
        };
        let ctx = InstallContext {
            root: registry::root_for(false),
            app_id,
            assoc,
            start_menu_dir: base.join("start-menu"),
            desktop_dir: base.join("desktop"),
            bundle: &bundle,
            version: "9.9.9",
        };

        let progress_log = std::cell::RefCell::new(Vec::new());
        run_install_with(&opts, &ctx, &|p| {
            progress_log.borrow_mut().push(p.clone());
        })
        .unwrap();

        // 首步应为关闭进程（目录已存在），且旧主程序被新包覆盖
        let log = progress_log.borrow();
        assert_eq!(log.first().unwrap().step, "close");
        assert!(log.last().unwrap().done);
        drop(log);
        assert_eq!(
            std::fs::read(install_dir.join(MAIN_EXE_NAME)).unwrap(),
            b"new main exe"
        );
        // 清理由 SiteCleanup 的 Drop 执行
    }
}
