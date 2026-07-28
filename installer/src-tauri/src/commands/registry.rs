// Copyright (c) 2026 青山公仔
// 轻壳 (Tauri Shell) is licensed under Mulan PSL v2.
// You can use this software according to the terms and conditions of the Mulan PSL v2.
// You may obtain a copy of Mulan PSL v2 at:
//          http://license.coscl.org.cn/MulanPSL2
// THIS SOFTWARE IS PROVIDED ON AN "AS IS" BASIS, WITHOUT WARRANTIES OF ANY KIND,
// EITHER EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO NON-INFRINGEMENT,
// MERCHANTABILITY OR FIT FOR A PARTICULAR PURPOSE.
// See the Mulan PSL v2 for more details.

//! 注册表操作：卸载信息写入/删除、文件关联登记面注册/反注册。
//!
//! root 作为参数传入，系统级/用户级安装通过 `root_for` 选择 HKLM/HKCU。

use winreg::enums::{HKEY_CURRENT_USER, HKEY_LOCAL_MACHINE, KEY_READ, KEY_WRITE};
use winreg::RegKey;

const UNINSTALL_PATH: &str = r"Software\Microsoft\Windows\CurrentVersion\Uninstall";
const CLASSES_PATH: &str = r"Software\Classes";
const REGISTERED_APPS_PATH: &str = r"Software\RegisteredApplications";

// 产品常量集中在 config.rs，此处再导出以保持既有引用路径
// （测试传入专用值以免触碰真实关联）
pub use crate::config::{APP_ID, ASSOC_EXTENSIONS, CAPABILITY_KEY, PROGID};

/// 根据安装类型选择注册表根键：系统级 HKLM，用户级 HKCU
pub fn root_for(is_system: bool) -> RegKey {
    if is_system {
        RegKey::predef(HKEY_LOCAL_MACHINE)
    } else {
        RegKey::predef(HKEY_CURRENT_USER)
    }
}

/// 控制面板"程序和功能"所需的卸载信息，字段与 NSIS 写入行为对齐
pub struct UninstallInfo {
    pub display_name: String,
    pub display_version: String,
    pub publisher: String,
    pub uninstall_string: String,
    pub install_location: String,
    pub display_icon: String,
    pub estimated_size_kb: u32,
}

fn set_str(key: &RegKey, name: &str, value: &str) -> Result<(), String> {
    key.set_value(name, &value)
        .map_err(|e| format!("写入注册表值 {name} 失败: {e}"))
}

fn set_dword(key: &RegKey, name: &str, value: u32) -> Result<(), String> {
    key.set_value(name, &value)
        .map_err(|e| format!("写入注册表值 {name} 失败: {e}"))
}

/// 删除子键树；键不存在视为成功（幂等）
fn delete_subkey_tolerant(root: &RegKey, path: &str) -> Result<(), String> {
    match root.delete_subkey_all(path) {
        Ok(()) => Ok(()),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(e) => Err(format!("删除注册表键 {path} 失败: {e}")),
    }
}

/// 写入控制面板卸载信息到 `{root}\Software\...\Uninstall\{app_id}`
pub fn write_uninstall_info(root: &RegKey, app_id: &str, info: &UninstallInfo) -> Result<(), String> {
    let path = format!(r"{UNINSTALL_PATH}\{app_id}");
    let (key, _) = root
        .create_subkey(&path)
        .map_err(|e| format!("创建注册表键 {path} 失败: {e}"))?;

    set_str(&key, "DisplayName", &info.display_name)?;
    set_str(&key, "DisplayVersion", &info.display_version)?;
    set_str(&key, "Publisher", &info.publisher)?;
    set_str(&key, "UninstallString", &info.uninstall_string)?;
    set_str(&key, "InstallLocation", &info.install_location)?;
    set_str(&key, "DisplayIcon", &info.display_icon)?;
    set_dword(&key, "EstimatedSize", info.estimated_size_kb)?;
    set_dword(&key, "NoModify", 1)?;
    set_dword(&key, "NoRepair", 1)?;
    Ok(())
}

/// 删除卸载信息（键不存在视为成功）
pub fn remove_uninstall_info(root: &RegKey, app_id: &str) -> Result<(), String> {
    delete_subkey_tolerant(root, &format!(r"{UNINSTALL_PATH}\{app_id}"))
}

/// 文件关联登记面的键名参数。progid/extensions 之外，进入系统
/// "打开方式/默认应用"候选列表还要求 Applications 与 Capabilities 登记；
/// 测试传专用值以免触碰真实注册表
#[derive(Clone, Copy)]
pub struct AssocParams<'a> {
    pub progid: &'a str,
    pub extensions: &'a [&'a str],
    /// `Software\Classes\Applications` 下的键名，须与主程序 exe 文件名一致
    pub exe_name: &'a str,
    /// Capabilities 父键（root 相对路径，如 `Software\MyApp`）
    pub capability_key: &'a str,
    /// RegisteredApplications 中的登记名
    pub app_name: &'a str,
}

/// 通知 shell 关联已变更，刷新资源管理器/设置页的关联缓存
/// （NSIS 模板同款机制，缺失时新登记面需等 Explorer 重启才可见）
fn notify_assoc_changed() {
    use windows::Win32::UI::Shell::{SHChangeNotify, SHCNE_ASSOCCHANGED, SHCNF_IDLIST};
    unsafe { SHChangeNotify(SHCNE_ASSOCCHANGED, SHCNF_IDLIST, None, None) };
}

/// 注册文件关联登记面：ProgID + Applications + Capabilities/RegisteredApplications
/// + 各扩展名 OpenWithProgids 候选。
///
/// 不写扩展名默认值：Win10/11 的 UserChoice 哈希封死程序改默认打开方式，
/// 完整登记让应用进入候选列表即可，设为默认由用户在系统 UI 完成。
pub fn register_file_associations(
    root: &RegKey,
    params: &AssocParams,
    exe_path: &str,
) -> Result<(), String> {
    let icon = format!("\"{exe_path}\",0");
    let open_command = format!("\"{exe_path}\" \"%1\"");

    // ProgID：关联的文件类型定义
    let progid_path = format!(r"{CLASSES_PATH}\{}", params.progid);
    let (progid_key, _) = root
        .create_subkey(&progid_path)
        .map_err(|e| format!("创建 ProgID 键失败: {e}"))?;
    set_str(&progid_key, "", crate::config::ASSOC_FILE_TYPE_NAME)?;
    let (icon_key, _) = root
        .create_subkey(format!(r"{progid_path}\DefaultIcon"))
        .map_err(|e| format!("创建 DefaultIcon 键失败: {e}"))?;
    set_str(&icon_key, "", &icon)?;
    let (cmd_key, _) = root
        .create_subkey(format!(r"{progid_path}\shell\open\command"))
        .map_err(|e| format!("创建 open command 键失败: {e}"))?;
    set_str(&cmd_key, "", &open_command)?;

    // Applications："打开方式"列表据此列出并解析本应用
    let app_path = format!(r"{CLASSES_PATH}\Applications\{}", params.exe_name);
    let (app_key, _) = root
        .create_subkey(&app_path)
        .map_err(|e| format!("创建 Applications 键失败: {e}"))?;
    set_str(&app_key, "FriendlyAppName", params.app_name)?;
    let (app_icon_key, _) = root
        .create_subkey(format!(r"{app_path}\DefaultIcon"))
        .map_err(|e| format!("创建 Applications DefaultIcon 键失败: {e}"))?;
    set_str(&app_icon_key, "", &icon)?;
    let (app_cmd_key, _) = root
        .create_subkey(format!(r"{app_path}\shell\open\command"))
        .map_err(|e| format!("创建 Applications command 键失败: {e}"))?;
    set_str(&app_cmd_key, "", &open_command)?;
    let (types_key, _) = root
        .create_subkey(format!(r"{app_path}\SupportedTypes"))
        .map_err(|e| format!("创建 SupportedTypes 键失败: {e}"))?;
    for ext in params.extensions {
        set_str(&types_key, &format!(".{ext}"), "")?;
    }

    // Capabilities + RegisteredApplications：进入系统"默认应用"设置页
    let cap_path = format!(r"{}\Capabilities", params.capability_key);
    let (cap_key, _) = root
        .create_subkey(&cap_path)
        .map_err(|e| format!("创建 Capabilities 键失败: {e}"))?;
    set_str(&cap_key, "ApplicationName", params.app_name)?;
    set_str(&cap_key, "ApplicationDescription", crate::config::APP_DESCRIPTION)?;
    let (assoc_key, _) = root
        .create_subkey(format!(r"{cap_path}\FileAssociations"))
        .map_err(|e| format!("创建 FileAssociations 键失败: {e}"))?;
    for ext in params.extensions {
        set_str(&assoc_key, &format!(".{ext}"), params.progid)?;
    }
    let (reg_apps_key, _) = root
        .create_subkey(REGISTERED_APPS_PATH)
        .map_err(|e| format!("打开 RegisteredApplications 失败: {e}"))?;
    set_str(&reg_apps_key, params.app_name, &cap_path)?;

    // OpenWithProgids：把 ProgID 挂入各扩展名的候选列表（不改默认值）
    for ext in params.extensions {
        let (progids_key, _) = root
            .create_subkey(format!(r"{CLASSES_PATH}\.{ext}\OpenWithProgids"))
            .map_err(|e| format!("创建 .{ext} OpenWithProgids 键失败: {e}"))?;
        set_str(&progids_key, params.progid, "")?;
    }

    notify_assoc_changed();
    Ok(())
}

/// 反注册文件关联登记面（对称删除，全部幂等）；
/// 不触碰其他应用的扩展名默认值与 OpenWithProgids 候选值
pub fn unregister_file_associations(root: &RegKey, params: &AssocParams) -> Result<(), String> {
    for ext in params.extensions {
        // 旧版安装器/手动设置可能把默认值指向本 ProgID，ProgID 删除后会悬空，
        // 仅当值等于自己时顺带清除（他人的默认值不动）
        if let Ok(ext_key) = root.open_subkey_with_flags(
            format!(r"{CLASSES_PATH}\.{ext}"),
            KEY_READ | KEY_WRITE,
        ) {
            if ext_key
                .get_value::<String, _>("")
                .is_ok_and(|v| v == params.progid)
            {
                let _ = ext_key.delete_value("");
            }
        }
        if let Ok(progids_key) = root.open_subkey_with_flags(
            format!(r"{CLASSES_PATH}\.{ext}\OpenWithProgids"),
            KEY_READ | KEY_WRITE,
        ) {
            let _ = progids_key.delete_value(params.progid); // 值不存在视为成功
        }
    }
    delete_subkey_tolerant(root, &format!(r"{CLASSES_PATH}\{}", params.progid))?;
    delete_subkey_tolerant(
        root,
        &format!(r"{CLASSES_PATH}\Applications\{}", params.exe_name),
    )?;
    delete_subkey_tolerant(root, params.capability_key)?;
    if let Ok(reg_apps_key) = root.open_subkey_with_flags(REGISTERED_APPS_PATH, KEY_READ | KEY_WRITE)
    {
        let _ = reg_apps_key.delete_value(params.app_name);
    }

    notify_assoc_changed();
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn hkcu() -> RegKey {
        // 测试统一走 root_for 的用户级分支，HKLM 需管理员权限不纳入自动化测试
        root_for(false)
    }

    #[test]
    fn uninstall_info_roundtrip() {
        let root = hkcu();
        let app_id = "MyAppWizardTest.Uninstall";
        let info = UninstallInfo {
            display_name: "My App".into(),
            display_version: "0.1.0".into(),
            publisher: "My Company".into(),
            uninstall_string: r#""C:\Fake\my-app-wizard.exe" --uninstall"#.into(),
            install_location: r"C:\Fake\My App".into(),
            display_icon: r"C:\Fake\My App\My App.exe".into(),
            estimated_size_kb: 12345,
        };

        write_uninstall_info(&root, app_id, &info).unwrap();

        let key = root
            .open_subkey(format!(r"{UNINSTALL_PATH}\{app_id}"))
            .unwrap();
        assert_eq!(key.get_value::<String, _>("DisplayName").unwrap(), "My App");
        assert_eq!(key.get_value::<String, _>("DisplayVersion").unwrap(), "0.1.0");
        assert_eq!(key.get_value::<String, _>("Publisher").unwrap(), "My Company");
        assert_eq!(
            key.get_value::<String, _>("UninstallString").unwrap(),
            r#""C:\Fake\my-app-wizard.exe" --uninstall"#
        );
        assert_eq!(
            key.get_value::<String, _>("InstallLocation").unwrap(),
            r"C:\Fake\My App"
        );
        assert_eq!(
            key.get_value::<String, _>("DisplayIcon").unwrap(),
            r"C:\Fake\My App\My App.exe"
        );
        assert_eq!(key.get_value::<u32, _>("EstimatedSize").unwrap(), 12345);
        assert_eq!(key.get_value::<u32, _>("NoModify").unwrap(), 1);
        assert_eq!(key.get_value::<u32, _>("NoRepair").unwrap(), 1);
        drop(key);

        remove_uninstall_info(&root, app_id).unwrap();
        assert!(root
            .open_subkey(format!(r"{UNINSTALL_PATH}\{app_id}"))
            .is_err());
        // 幂等：重复删除不报错
        remove_uninstall_info(&root, app_id).unwrap();
    }

    /// 登记面现场清理（复用反注册的幂等性，测试前后各调一次自愈残留）
    fn clean_assoc_site(root: &RegKey, params: &AssocParams) {
        let _ = unregister_file_associations(root, params);
        for ext in params.extensions {
            let _ = root.delete_subkey_all(format!(r"{CLASSES_PATH}\.{ext}"));
        }
    }

    #[test]
    fn file_associations_register_full_surface_and_unregister() {
        let root = hkcu();
        let params = AssocParams {
            progid: "MyAppWizardTest.ProgId1",
            extensions: &["myapptest1"],
            exe_name: "myapp-wizard-test1.exe",
            capability_key: r"Software\MyAppWizardTest.Caps1",
            app_name: "MyAppWizardTest App1",
        };
        let ext = params.extensions[0];
        let ext_path = format!(r"{CLASSES_PATH}\.{ext}");
        let exe = r"C:\Fake\My App\my-app.exe";
        clean_assoc_site(&root, &params);

        // 预置"他人"的默认值与候选值：注册/反注册全程不得触碰
        let (ext_key, _) = root.create_subkey(&ext_path).unwrap();
        ext_key.set_value("", &"Legacy.ProgID").unwrap();
        drop(ext_key);
        let (progids_key, _) = root
            .create_subkey(format!(r"{ext_path}\OpenWithProgids"))
            .unwrap();
        progids_key.set_value("Other.ProgID", &"").unwrap();
        drop(progids_key);

        register_file_associations(&root, &params, exe).unwrap();

        // ProgID 打开命令
        let cmd_key = root
            .open_subkey(format!(
                r"{CLASSES_PATH}\{}\shell\open\command",
                params.progid
            ))
            .unwrap();
        assert_eq!(
            cmd_key.get_value::<String, _>("").unwrap(),
            format!("\"{exe}\" \"%1\"")
        );
        drop(cmd_key);

        // Applications：FriendlyAppName + 打开命令 + SupportedTypes
        let app_path = format!(r"{CLASSES_PATH}\Applications\{}", params.exe_name);
        let app_key = root.open_subkey(&app_path).unwrap();
        assert_eq!(
            app_key.get_value::<String, _>("FriendlyAppName").unwrap(),
            params.app_name
        );
        drop(app_key);
        let app_cmd_key = root
            .open_subkey(format!(r"{app_path}\shell\open\command"))
            .unwrap();
        assert_eq!(
            app_cmd_key.get_value::<String, _>("").unwrap(),
            format!("\"{exe}\" \"%1\"")
        );
        drop(app_cmd_key);
        let types_key = root
            .open_subkey(format!(r"{app_path}\SupportedTypes"))
            .unwrap();
        assert_eq!(
            types_key.get_value::<String, _>(format!(".{ext}")).unwrap(),
            ""
        );
        drop(types_key);

        // Capabilities + RegisteredApplications
        let cap_path = format!(r"{}\Capabilities", params.capability_key);
        let cap_key = root.open_subkey(&cap_path).unwrap();
        assert_eq!(
            cap_key.get_value::<String, _>("ApplicationName").unwrap(),
            params.app_name
        );
        drop(cap_key);
        let assoc_key = root
            .open_subkey(format!(r"{cap_path}\FileAssociations"))
            .unwrap();
        assert_eq!(
            assoc_key.get_value::<String, _>(format!(".{ext}")).unwrap(),
            params.progid
        );
        drop(assoc_key);
        let reg_apps = root.open_subkey(REGISTERED_APPS_PATH).unwrap();
        assert_eq!(
            reg_apps.get_value::<String, _>(params.app_name).unwrap(),
            cap_path
        );
        drop(reg_apps);

        // OpenWithProgids 挂上候选，且扩展名默认值未被触碰
        let ext_key = root.open_subkey(&ext_path).unwrap();
        assert_eq!(ext_key.get_value::<String, _>("").unwrap(), "Legacy.ProgID");
        drop(ext_key);
        let progids_key = root
            .open_subkey(format!(r"{ext_path}\OpenWithProgids"))
            .unwrap();
        assert_eq!(
            progids_key.get_value::<String, _>(params.progid).unwrap(),
            ""
        );
        drop(progids_key);

        unregister_file_associations(&root, &params).unwrap();

        // 登记面全部消失；他人的默认值与候选值原样保留
        assert!(root
            .open_subkey(format!(r"{CLASSES_PATH}\{}", params.progid))
            .is_err());
        assert!(root.open_subkey(&app_path).is_err());
        assert!(root.open_subkey(params.capability_key).is_err());
        let reg_apps = root.open_subkey(REGISTERED_APPS_PATH).unwrap();
        assert!(reg_apps.get_value::<String, _>(params.app_name).is_err());
        drop(reg_apps);
        let ext_key = root.open_subkey(&ext_path).unwrap();
        assert_eq!(ext_key.get_value::<String, _>("").unwrap(), "Legacy.ProgID");
        drop(ext_key);
        let progids_key = root
            .open_subkey(format!(r"{ext_path}\OpenWithProgids"))
            .unwrap();
        assert!(progids_key.get_value::<String, _>(params.progid).is_err());
        assert_eq!(
            progids_key.get_value::<String, _>("Other.ProgID").unwrap(),
            ""
        );
        drop(progids_key);

        clean_assoc_site(&root, &params);
    }

    #[test]
    fn unregister_clears_dangling_own_default_value() {
        let root = hkcu();
        let params = AssocParams {
            progid: "MyAppWizardTest.ProgId4",
            extensions: &["myapptest4"],
            exe_name: "myapp-wizard-test4.exe",
            capability_key: r"Software\MyAppWizardTest.Caps4",
            app_name: "MyAppWizardTest App4",
        };
        let ext_path = format!(r"{CLASSES_PATH}\.{}", params.extensions[0]);
        clean_assoc_site(&root, &params);

        // 模拟旧版安装器把默认值指向本 ProgID 的残留现场
        let (ext_key, _) = root.create_subkey(&ext_path).unwrap();
        ext_key.set_value("", &params.progid).unwrap();
        drop(ext_key);

        unregister_file_associations(&root, &params).unwrap();

        // 指向自身的默认值被清除，不留悬空 ProgID
        let ext_key = root.open_subkey(&ext_path).unwrap();
        assert_eq!(ext_key.get_value::<String, _>("").unwrap_or_default(), "");
        drop(ext_key);

        clean_assoc_site(&root, &params);
    }

    #[test]
    fn unregister_is_idempotent_on_empty_site() {
        let root = hkcu();
        let params = AssocParams {
            progid: "MyAppWizardTest.ProgId3",
            extensions: &["myapptest3"],
            exe_name: "myapp-wizard-test3.exe",
            capability_key: r"Software\MyAppWizardTest.Caps3",
            app_name: "MyAppWizardTest App3",
        };
        // 空现场反复反注册不报错（卸载容错依赖此幂等性）
        unregister_file_associations(&root, &params).unwrap();
        unregister_file_associations(&root, &params).unwrap();
    }
}
