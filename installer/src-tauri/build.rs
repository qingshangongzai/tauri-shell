// Copyright (c) 2026 青山公仔
// 轻壳 (Tauri Shell) is licensed under Mulan PSL v2.
// You can use this software according to the terms and conditions of the Mulan PSL v2.
// You may obtain a copy of Mulan PSL v2 at:
//          http://license.coscl.org.cn/MulanPSL2
// THIS SOFTWARE IS PROVIDED ON AN "AS IS" BASIS, WITHOUT WARRANTIES OF ANY KIND,
// EITHER EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO NON-INFRINGEMENT,
// MERCHANTABILITY OR FIT FOR A PARTICULAR PURPOSE.
// See the Mulan PSL v2 for more details.

use std::path::Path;

/// 主应用打包产物（仓库根，由 scripts/build-installer.mjs 产出）
const BUNDLE_SOURCE: &str = "../../app-bundle.zip";

fn main() {
    // 将 bundle 复制到 OUT_DIR 供 include_bytes! 嵌入；
    // 缺失时写零字节占位，保证开发/测试期可编译（运行时报"安装包未嵌入"）
    let out_dir = std::env::var("OUT_DIR").expect("OUT_DIR 不可用");
    let dest = Path::new(&out_dir).join("app-bundle.zip");
    // 有意为之的取舍：bundle 不存在时 Cargo 对缺失路径会每次重跑 build script
    // （拖慢开发期构建），但能保证打包脚本产出 bundle 后自动重新嵌入，
    // 避免静默嵌入陈旧占位字节产出坏安装器
    println!("cargo:rerun-if-changed={BUNDLE_SOURCE}");
    let source = Path::new(BUNDLE_SOURCE);
    if source.exists() {
        std::fs::copy(source, &dest).expect("复制 app-bundle.zip 到 OUT_DIR 失败");
    } else {
        std::fs::write(&dest, []).expect("写入占位 bundle 失败");
    }

    tauri_build::build();
}
