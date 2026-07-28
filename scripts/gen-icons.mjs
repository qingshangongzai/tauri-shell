// Copyright (c) 2026 青山公仔
// 轻壳 (Tauri Shell) is licensed under Mulan PSL v2.
// You can use this software according to the terms and conditions of the Mulan PSL v2.
// You may obtain a copy of Mulan PSL v2 at:
//          http://license.coscl.org.cn/MulanPSL2
// THIS SOFTWARE IS PROVIDED ON AN "AS IS" BASIS, WITHOUT WARRANTIES OF ANY KIND,
// EITHER EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO NON-INFRINGEMENT,
// MERCHANTABILITY OR FIT FOR A PARTICULAR PURPOSE.
// See the Mulan PSL v2 for more details.

// 图标生成（beforeBuildCommand 自动调用）：
// 全项目只维护一张源图 src-tauri/icons/logo.svg，本脚本用 tauri icon
// 生成图标后只保留 Windows 所需的 icon.ico，Android/iOS/Store 等
// 无关平台的生成物一律清除，并把源图与 ico 同步给安装器子包。
//
// 注意：src-tauri/icons/ 目录由本脚本管理，logo.svg 与 icon.ico
// 之外的文件每次运行都会被删除，请勿在该目录手工存放其它文件。
import { execSync } from "node:child_process";
import {
  copyFileSync,
  existsSync,
  readdirSync,
  rmSync,
} from "node:fs";
import { join, resolve, dirname } from "node:path";
import { fileURLToPath } from "node:url";

const root = resolve(dirname(fileURLToPath(import.meta.url)), "..");
const iconsDir = join(root, "src-tauri", "icons");
const KEEP = new Set(["logo.svg", "icon.ico"]);

// tauri icon 会生成全平台图标（无平台过滤选项），生成后按 KEEP 清理
execSync("npx tauri icon src-tauri/icons/logo.svg", {
  cwd: root,
  stdio: "inherit",
});
for (const entry of readdirSync(iconsDir)) {
  if (!KEEP.has(entry)) {
    rmSync(join(iconsDir, entry), { recursive: true, force: true });
  }
}

// 同步给安装器子包：向导页面 logo + 安装器 EXE 图标（子包被裁剪时跳过）
if (existsSync(join(root, "installer"))) {
  copyFileSync(
    join(iconsDir, "logo.svg"),
    join(root, "installer", "public", "logo.svg"),
  );
  copyFileSync(
    join(iconsDir, "icon.ico"),
    join(root, "installer", "src-tauri", "icons", "icon.ico"),
  );
}

console.log("Icons generated: icon.ico (from logo.svg), extras removed");
