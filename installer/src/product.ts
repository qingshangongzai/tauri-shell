// Copyright (c) 2026 青山公仔
// 轻壳 (Tauri Shell) is licensed under Mulan PSL v2.
// You can use this software according to the terms and conditions of the Mulan PSL v2.
// You may obtain a copy of Mulan PSL v2 at:
//          http://license.coscl.org.cn/MulanPSL2
// THIS SOFTWARE IS PROVIDED ON AN "AS IS" BASIS, WITHOUT WARRANTIES OF ANY KIND,
// EITHER EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO NON-INFRINGEMENT,
// MERCHANTABILITY OR FIT FOR A PARTICULAR PURPOSE.
// See the Mulan PSL v2 for more details.

/**
 * 产品文案配置：前端向导的全部产品相关文案集中于此。
 * 模板使用者改名时只需修改本文件（以及 Rust 侧的 src-tauri/src/config.rs）。
 */

/** 产品显示名（须与 config.rs 的 APP_DISPLAY_NAME 一致） */
export const PRODUCT_NAME = "我的应用";

/** 欢迎页产品简介 */
export const PRODUCT_TAGLINE = `${PRODUCT_NAME} 是一款基于轻壳构建的桌面应用。本向导将引导你完成安装。`;

/** 应用 ID（须与 config.rs 的 APP_ID 一致），用户数据位于 %APPDATA%\{APP_ID} */
export const APP_ID = "com.mycompany.myapp";

/**
 * 是否在选项页展示"文件关联"开关。
 * 须与 config.rs 的 ASSOC_EXTENSIONS 联动：扩展名列表非空时才设为 true。
 */
export const FILE_ASSOC_ENABLED = false;

/** 文件关联开关的标题与描述（FILE_ASSOC_ENABLED 为 true 时展示） */
export const FILE_ASSOC_LABEL = "关联文件类型";
export const FILE_ASSOC_DESCRIPTION = `将 ${PRODUCT_NAME} 注册为相关文件的打开方式，可在系统设置中设为默认应用。`;
