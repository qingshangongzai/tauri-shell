// Copyright (c) 2026 青山公仔
// 轻壳 (Tauri Shell) is licensed under Mulan PSL v2.
// You can use this software according to the terms and conditions of the Mulan PSL v2.
// You may obtain a copy of Mulan PSL v2 at:
//          http://license.coscl.org.cn/MulanPSL2
// THIS SOFTWARE IS PROVIDED ON AN "AS IS" BASIS, WITHOUT WARRANTIES OF ANY KIND,
// EITHER EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO NON-INFRINGEMENT,
// MERCHANTABILITY OR FIT FOR A PARTICULAR PURPOSE.
// See the Mulan PSL v2 for more details.

// 托盘 opt-in 构建期开关的可靠性测试：node scripts/test-tray-opt-in.mjs（或 npm run test:tray）
//
// 验证内容：
//   用例 A「无标记」— dist/index.html 不带 data-tauri-tray →
//     · minify.cjs 不打包 tray-menu.html（dist-min 无该文件）
//     · build.rs 不输出 tray_enabled → 托盘代码不编译，且 lib.rs 一致性测试
//       （tray_cfg_matches_html_marker）断言 cfg off == 标记 off
//   用例 B「有标记」— dist/index.html 带 data-tauri-tray →
//     · minify.cjs 打包 tray-menu.html（dist-min 有该文件）
//     · build.rs 输出 tray_enabled → 托盘代码编译通过，且一致性测试断言 cfg on == 标记 on
// 最后自动还原 dist/index.html 与 dist-min/。
//
// 注意：首次运行需编译 Rust 依赖（cargo test），耗时数分钟属正常。

import { execFileSync } from "node:child_process";
import { existsSync, readFileSync, writeFileSync } from "node:fs";
import { fileURLToPath } from "node:url";
import path from "node:path";

const ROOT = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const INDEX = path.join(ROOT, "dist", "index.html");
const DIST_MIN = path.join(ROOT, "dist-min");
const MANIFEST = path.join(ROOT, "src-tauri", "Cargo.toml");
const TEST_NAME = "tray_cfg_matches_html_marker";
const CARGO_TIMEOUT = 15 * 60 * 1000; // 首次编译依赖较慢，放宽到 15 分钟

const NO_MARKER_HTML =
  '<!DOCTYPE html><html lang="zh-CN"><head><meta charset="UTF-8"><title>no-tray</title></head><body>no tray</body></html>';
const WITH_MARKER_HTML =
  '<!DOCTYPE html><html lang="zh-CN" data-tauri-tray><head><meta charset="UTF-8"><title>with-tray</title></head><body>with tray</body></html>';

function run(cmd, args, timeout = 60_000) {
  try {
    const stdout = execFileSync(cmd, args, {
      cwd: ROOT,
      encoding: "utf8",
      timeout,
      stdio: ["ignore", "pipe", "pipe"],
    });
    return { ok: true, stdout };
  } catch (e) {
    return { ok: false, stdout: `${e.stdout ?? ""}${e.stderr ?? ""}`, code: e.status };
  }
}

const results = [];
function check(name, cond, detail = "") {
  results.push(!!cond);
  console.log(`${cond ? "PASS" : "FAIL"}  ${name}${detail ? `  → ${detail}` : ""}`);
}

// 跑 minify.cjs：失败时打印输出尾部，便于直接定位原因
function minifyOk() {
  const r = run(process.execPath, ["minify.cjs"]);
  if (!r.ok) console.log(r.stdout.slice(-1000));
  return r.ok;
}

// 跑 cargo test：退出码 0 且一致性测试确实执行（输出含测试名）才判通过
function cargoTestOk(label) {
  console.log(`\n[${label}] cargo test（首次运行编译依赖，请耐心等待…）`);
  const r = run("cargo", ["test", "--manifest-path", MANIFEST], CARGO_TIMEOUT);
  if (!r.ok) {
    console.log(r.stdout.slice(-3000));
    return false;
  }
  if (!r.stdout.includes(TEST_NAME)) {
    console.log("警告：cargo test 通过但输出中未出现一致性测试，视为失败\n" + r.stdout.slice(-1000));
    return false;
  }
  return true;
}

const backup = existsSync(INDEX) ? readFileSync(INDEX, "utf8") : null;
try {
  // ── 用例 A：无标记 ──
  console.log("════════ 用例 A：无 data-tauri-tray 标记（默认不打包托盘） ════════");
  writeFileSync(INDEX, NO_MARKER_HTML);
  check("A1 minify.cjs 运行成功", minifyOk());
  check("A2 dist-min 无 tray-menu.html", !existsSync(path.join(DIST_MIN, "tray-menu.html")));
  check("A3 dist-min 有 index.html", existsSync(path.join(DIST_MIN, "index.html")));
  check("A4 无标记下编译与一致性测试通过（cfg off == 标记 off）", cargoTestOk("用例 A"));

  // ── 用例 B：有标记 ──
  console.log("\n════════ 用例 B：带 data-tauri-tray 标记（打包托盘） ════════");
  writeFileSync(INDEX, WITH_MARKER_HTML);
  check("B1 minify.cjs 运行成功", minifyOk());
  check("B2 dist-min 有 tray-menu.html", existsSync(path.join(DIST_MIN, "tray-menu.html")));
  check("B3 有标记下托盘代码编译通过（cfg on == 标记 on）", cargoTestOk("用例 B"));
} finally {
  // ── 还原：恢复 dist/index.html 并重跑 minify.cjs 还原 dist-min ──
  if (backup !== null) writeFileSync(INDEX, backup);
  console.log("\n[还原] 已恢复 dist/index.html，重跑 minify.cjs 还原 dist-min/…");
  run(process.execPath, ["minify.cjs"]);
}

console.log(`\n════════ 结果：${results.filter(Boolean).length}/${results.length} 项通过 ════════`);
process.exit(results.every(Boolean) ? 0 : 1);
