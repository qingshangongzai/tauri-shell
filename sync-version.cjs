// Copyright (c) 2026 青山公仔
// 轻壳 (Tauri Shell) is licensed under Mulan PSL v2.
// You can use this software according to the terms and conditions of the Mulan PSL v2.
// You may obtain a copy of Mulan PSL v2 at:
//          http://license.coscl.org.cn/MulanPSL2
// THIS SOFTWARE IS PROVIDED ON AN "AS IS" BASIS, WITHOUT WARRANTIES OF ANY KIND,
// EITHER EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO NON-INFRINGEMENT,
// MERCHANTABILITY OR FIT FOR A PARTICULAR PURPOSE.
// See the Mulan PSL v2 for more details.

const fs = require("fs");
const path = require("path");

// 读取 tauri.conf.json 中的 version
const tauriConf = JSON.parse(fs.readFileSync("src-tauri/tauri.conf.json", "utf8"));
const version = tauriConf.version;

// 同步到 package.json
function syncPackageJson(pkgPath) {
  const pkg = JSON.parse(fs.readFileSync(pkgPath, "utf8"));
  pkg.version = version;
  fs.writeFileSync(pkgPath, JSON.stringify(pkg, null, 2) + "\n");
}

// 同步到 Cargo.toml / tauri.conf.json（正则替换首个 version 字段，保留原格式）
function syncByRegex(filePath, pattern, replacement) {
  let text = fs.readFileSync(filePath, "utf8");
  text = text.replace(pattern, replacement);
  fs.writeFileSync(filePath, text);
}

syncPackageJson("package.json");
syncByRegex(
  path.join("src-tauri", "Cargo.toml"),
  /^version\s*=\s*".*"/m,
  `version = "${version}"`
);

// 同步到安装器子包（安装器版本号与主应用保持一致；
// 子包被裁剪掉时跳过，不阻断 NSIS 等其它产线的构建）
if (fs.existsSync("installer")) {
  syncPackageJson(path.join("installer", "package.json"));
  syncByRegex(
    path.join("installer", "src-tauri", "Cargo.toml"),
    /^version\s*=\s*".*"/m,
    `version = "${version}"`
  );
  syncByRegex(
    path.join("installer", "src-tauri", "tauri.conf.json"),
    /"version"\s*:\s*"[^"]*"/,
    `"version": "${version}"`
  );
}

console.log(`Version synced: ${version}`);
