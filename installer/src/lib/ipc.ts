// Copyright (c) 2026 青山公仔
// 轻壳 (Tauri Shell) is licensed under Mulan PSL v2.
// You can use this software according to the terms and conditions of the Mulan PSL v2.
// You may obtain a copy of Mulan PSL v2 at:
//          http://license.coscl.org.cn/MulanPSL2
// THIS SOFTWARE IS PROVIDED ON AN "AS IS" BASIS, WITHOUT WARRANTIES OF ANY KIND,
// EITHER EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO NON-INFRINGEMENT,
// MERCHANTABILITY OR FIT FOR A PARTICULAR PURPOSE.
// See the Mulan PSL v2 for more details.

import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";

/** 后端 get_install_config 返回（camelCase IPC 契约，见 src-tauri/src/lib.rs） */
export interface InstallConfig {
  version: string;
  systemDefaultDir: string;
  userDefaultDir: string;
  /** 向导模式，前端据此分支渲染安装/卸载流程 */
  mode: "install" | "uninstall";
  /** 卸载模式的现场信息；安装模式或未检测到安装时为 null */
  uninstallInfo: UninstallInfo | null;
}

/** 检测到的卸载现场（对应后端 UninstallSite） */
export interface UninstallInfo {
  isSystem: boolean;
  installDir: string;
  createdDesktopShortcut: boolean;
}

/** start_install 入参（对应后端 InstallOptions） */
export interface InstallOptions {
  isSystem: boolean;
  installDir: string;
  desktopShortcut: boolean;
  fileAssoc: boolean;
}

/** install://progress 事件负载（对应后端 InstallProgress） */
export interface InstallProgress {
  step: string;
  percent: number;
  message: string;
  done: boolean;
  error: string | null;
}

export function getInstallConfig(): Promise<InstallConfig> {
  return invoke<InstallConfig>("get_install_config");
}

/** 目标目录下是否有运行中的进程（主程序/旧卸载器） */
export function isAppRunning(installDir: string): Promise<boolean> {
  return invoke<boolean>("is_app_running", { installDir });
}

export function startInstall(options: InstallOptions): Promise<void> {
  return invoke("start_install", { options });
}

/** start_uninstall 入参（对应后端 UninstallUiOptions） */
export interface UninstallOptions {
  removeUserData: boolean;
}

export function startUninstall(options: UninstallOptions): Promise<void> {
  return invoke("start_uninstall", { options });
}

export function launchApp(installDir: string): Promise<void> {
  return invoke("launch_app", { installDir });
}

/** 深链直达系统设置中本应用的"默认应用"页（设为默认只能由用户在系统 UI 完成） */
export function openDefaultAppsSettings(isSystem: boolean): Promise<void> {
  return invoke("open_default_apps_settings", { isSystem });
}

export function onInstallProgress(
  handler: (progress: InstallProgress) => void,
): Promise<UnlistenFn> {
  return listen<InstallProgress>("install://progress", (e) =>
    handler(e.payload),
  );
}
