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
const pkgPath = "package.json";
const pkg = JSON.parse(fs.readFileSync(pkgPath, "utf8"));
pkg.version = version;
fs.writeFileSync(pkgPath, JSON.stringify(pkg, null, 2) + "\n");

// 同步到 Cargo.toml（使用正则替换）
const cargoPath = path.join("src-tauri", "Cargo.toml");
let cargo = fs.readFileSync(cargoPath, "utf8");
cargo = cargo.replace(
  /^version\s*=\s*".*"/m,
  `version = "${version}"`
);
fs.writeFileSync(cargoPath, cargo);

console.log(`Version synced: ${version}`);
