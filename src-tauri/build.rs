// Copyright (c) 2026 青山公仔
// 轻壳 (Tauri Shell) is licensed under Mulan PSL v2.
// You can use this software according to the terms and conditions of the Mulan PSL v2.
// You may obtain a copy of Mulan PSL v2 at:
//          http://license.coscl.org.cn/MulanPSL2
// THIS SOFTWARE IS PROVIDED ON AN "AS IS" BASIS, WITHOUT WARRANTIES OF ANY KIND,
// EITHER EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO NON-INFRINGEMENT,
// MERCHANTABILITY OR FIT FOR A PARTICULAR PURPOSE.
// See the Mulan PSL v2 for more details.

// 托盘 opt-in 开关：dist/index.html 的 <html> 带 data-tauri-tray 属性才编译托盘代码，
// 无标记则默认不打包托盘、关闭主窗口直接退出。改标记后 rerun-if-changed 自动触发增量重编。
fn main() {
    let marker = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../dist/index.html");
    println!("cargo:rerun-if-changed={}", marker.display());
    println!("cargo::rustc-check-cfg=cfg(tray_enabled)");
    if std::fs::read_to_string(&marker)
        .unwrap_or_default()
        .contains("data-tauri-tray")
    {
        println!("cargo:rustc-cfg=tray_enabled");
    }
    tauri_build::build()
}
