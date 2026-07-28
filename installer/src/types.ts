// Copyright (c) 2026 青山公仔
// 轻壳 (Tauri Shell) is licensed under Mulan PSL v2.
// You can use this software according to the terms and conditions of the Mulan PSL v2.
// You may obtain a copy of Mulan PSL v2 at:
//          http://license.coscl.org.cn/MulanPSL2
// THIS SOFTWARE IS PROVIDED ON AN "AS IS" BASIS, WITHOUT WARRANTIES OF ANY KIND,
// EITHER EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO NON-INFRINGEMENT,
// MERCHANTABILITY OR FIT FOR A PARTICULAR PURPOSE.
// See the Mulan PSL v2 for more details.

/** 安装向导步骤（顺序即流程顺序） */
export const INSTALL_STEPS = [
  "welcome",
  "path",
  "options",
  "progress",
  "finish",
] as const;

export type InstallStep = (typeof INSTALL_STEPS)[number];

export const INSTALL_STEP_LABELS: Record<InstallStep, string> = {
  welcome: "欢迎",
  path: "位置",
  options: "选项",
  progress: "安装",
  finish: "完成",
};

/** 卸载向导步骤 */
export const UNINSTALL_STEPS = ["confirm", "progress", "finish"] as const;

export type UninstallStep = (typeof UNINSTALL_STEPS)[number];

export const UNINSTALL_STEP_LABELS: Record<UninstallStep, string> = {
  confirm: "确认",
  progress: "卸载",
  finish: "完成",
};
