// Copyright (c) 2026 青山公仔
// 轻壳 (Tauri Shell) is licensed under Mulan PSL v2.
// You can use this software according to the terms and conditions of the Mulan PSL v2.
// You may obtain a copy of Mulan PSL v2 at:
//          http://license.coscl.org.cn/MulanPSL2
// THIS SOFTWARE IS PROVIDED ON AN "AS IS" BASIS, WITHOUT WARRANTIES OF ANY KIND,
// EITHER EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO NON-INFRINGEMENT,
// MERCHANTABILITY OR FIT FOR A PARTICULAR PURPOSE.
// See the Mulan PSL v2 for more details.

//! 产品配置中心：安装器的全部产品常量集中于此。
//!
//! 模板使用者改名时只需修改本文件（以及前端的 `src/product.ts`）即可，
//! 无需触碰安装/卸载编排代码。需要与宿主项目保持一致的字段：
//!
//! - `MAIN_EXE_NAME`：主项目 `src-tauri/Cargo.toml` 的 `[[bin]] name` + `.exe`
//! - `WIZARD_EXE_NAME`：本包 `src-tauri/Cargo.toml` 的 `[[bin]] name` + `.exe`
//! - `APP_ID`：建议与主项目 `tauri.conf.json` 的 `identifier` 一致

/// 嵌入 zip 根目录的主程序 exe 名（与 scripts/build-installer.mjs 打包约定一致）
pub const MAIN_EXE_NAME: &str = "my-app.exe";
/// 复制到安装目录的卸载器 exe 名（即安装器自身）
pub const WIZARD_EXE_NAME: &str = "my-app-wizard.exe";
/// 控制面板与快捷方式显示名称
pub const APP_DISPLAY_NAME: &str = "我的应用";
/// 发布者（控制面板"发布者"列）
pub const PUBLISHER: &str = "My Company";
/// 应用 ID：注册表卸载键名，也是用户数据目录名（%APPDATA%\{APP_ID}）
pub const APP_ID: &str = "com.mycompany.myapp";
/// 安装目录与开始菜单程序组的文件夹名
pub const INSTALL_DIR_NAME: &str = "我的应用";
/// %TEMP% 下日志与进度文件的前缀（{前缀}.log / {前缀}-progress-*.json）
pub const TEMP_PREFIX: &str = "my-app-wizard";

// ---- 文件关联（可选功能）----
// 扩展名列表为空时安装编排自动跳过文件关联步骤。需要启用时：
// 填入扩展名，并把前端 `src/product.ts` 的 FILE_ASSOC_ENABLED 改为 true。

/// 要登记的扩展名列表（不带点，如 &["md", "markdown"]）；空 = 关闭文件关联
pub const ASSOC_EXTENSIONS: &[&str] = &[];
/// 文件关联 ProgID
pub const PROGID: &str = "MyApp.file";
/// Capabilities 父键（注册表 root 相对路径）
pub const CAPABILITY_KEY: &str = r"Software\MyApp";
/// ProgID 默认值（资源管理器中显示的文件类型描述）
pub const ASSOC_FILE_TYPE_NAME: &str = "我的应用 文档";
/// Capabilities 的 ApplicationDescription（系统"默认应用"设置页描述）
pub const APP_DESCRIPTION: &str = "基于轻壳构建的桌面应用";
