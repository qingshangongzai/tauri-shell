// Copyright (c) 2026 青山公仔
// 轻壳 (Tauri Shell) is licensed under Mulan PSL v2.
// You can use this software according to the terms and conditions of the Mulan PSL v2.
// You may obtain a copy of Mulan PSL v2 at:
//          http://license.coscl.org.cn/MulanPSL2
// THIS SOFTWARE IS PROVIDED ON AN "AS IS" BASIS, WITHOUT WARRANTIES OF ANY KIND,
// EITHER EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO NON-INFRINGEMENT,
// MERCHANTABILITY OR FIT FOR A PARTICULAR PURPOSE.
// See the Mulan PSL v2 for more details.

// 自定义安装器打包链（双产物流程，替代 NSIS）：
//   1. 主应用构建（tauri build --no-bundle，beforeBuildCommand 已含 sync-version/gen-icons/minify）
//   2. 编译安装器（前端 build + 第一次 cargo build），
//      build.rs 占位机制生效，产出"纯壳"向导 exe（不含 zip，即卸载器）
//   3. 主程序 exe + 纯壳卸载器压缩为仓库根 app-bundle.zip
//   4. 第二次 cargo build（参数与第一次完全一致）嵌入 zip，产出完整安装器
//   5. 收集产物到 dist-installer/{productName}_{version}_x64-setup.exe 并校验体积
//
// 产品信息不在本脚本硬编码：主程序 exe 名/向导 exe 名解析自两侧 Cargo.toml
// 的 [[bin]] name，产物名与版本号读自 src-tauri/tauri.conf.json。
//
// 参数：--skip-main 跳过第 1 步，复用已有的主程序 exe
// （仅改动安装器时的快速迭代，主应用有改动时勿用）
import { execSync } from "node:child_process";
import {
  copyFileSync,
  existsSync,
  mkdirSync,
  mkdtempSync,
  readFileSync,
  rmSync,
  statSync,
} from "node:fs";
import { tmpdir } from "node:os";
import { join, resolve, dirname } from "node:path";
import { fileURLToPath } from "node:url";

const root = resolve(dirname(fileURLToPath(import.meta.url)), "..");
const skipMain = process.argv.includes("--skip-main");
const bundleZip = join(root, "app-bundle.zip");
const installerDir = join(root, "installer");
const installerCrateDir = join(installerDir, "src-tauri");

// 从 Cargo.toml 解析 [[bin]] name（模板使用者改名后本脚本无需修改）
function readBinName(cargoTomlPath) {
  const toml = readFileSync(cargoTomlPath, "utf-8");
  const match = toml.match(/\[\[bin\]\][^[]*?name\s*=\s*"([^"]+)"/);
  if (!match) {
    fail(`cannot find [[bin]] name in ${cargoTomlPath}`);
  }
  return match[1];
}

const mainBin = readBinName(join(root, "src-tauri", "Cargo.toml"));
const wizardBin = readBinName(join(installerCrateDir, "Cargo.toml"));
const wizardExe = join(
  installerCrateDir,
  "target",
  "release",
  `${wizardBin}.exe`,
);

// 两次编译参数必须完全一致，否则 Cargo 视为不同配置大面积重编。
// 不走 tauri build：其会按 productName 重命名二进制（含中文），
// 直接 cargo build 产出确定命名的向导 exe（build.rs 负责嵌入 zip）。
// 必须显式开启 tauri/custom-protocol（tauri build 会自动附加）：
// tauri 的 dev/prod 由该 feature 而非 --release 决定，缺失时 cfg(dev) 生效，
// 产物会去加载 devUrl（localhost:1430）而非嵌入的 frontendDist
const CARGO_BUILD_CMD =
  "cargo build --release --features tauri/custom-protocol";

// 日志一律 ASCII：Node 输出 UTF-8，而 PowerShell 控制台/管道默认按 GBK 解码，
// 中文日志会显示为乱码
const TOTAL_STEPS = 5;
let stepNo = 0;
const startedAt = Date.now();

function elapsed() {
  return `${((Date.now() - startedAt) / 1000).toFixed(0)}s`;
}

function step(title) {
  stepNo += 1;
  console.log(
    `\n[build-installer] ===== [${stepNo}/${TOTAL_STEPS}] ${title} (${elapsed()}) =====`,
  );
}

function fail(message) {
  console.error(`[build-installer] ERROR: ${message}`);
  process.exit(1);
}

function run(cmd, cwd = root) {
  console.log(`[build-installer] $ ${cmd}`);
  // stdio inherit：cargo/vite 的编译过程日志直接透传到当前终端
  execSync(cmd, { cwd, stdio: "inherit" });
}

function fmtMB(bytes) {
  return `${(bytes / 1024 / 1024).toFixed(1)} MB`;
}

// ---- 1. 主应用构建 ----
step("build main app (tauri build --no-bundle)");
if (skipMain) {
  // 快速迭代：跳过主应用构建，直接复用上次产物（下方仍校验 exe 存在）
  console.log("[build-installer] --skip-main: reuse existing main exe");
} else {
  run("npm run tauri -- build --no-bundle");
}

// --no-bundle 不执行 bundle 阶段的 productName 重命名，
// 二进制固定为 cargo 产出的 {mainBin}.exe，缺失直接失败
const releaseDir = join(root, "src-tauri", "target", "release");
const mainExe = join(releaseDir, `${mainBin}.exe`);
if (!existsSync(mainExe)) {
  fail(`main app exe not found: ${mainExe}`);
}
console.log(
  `[build-installer] main exe: ${mainExe} (${fmtMB(statSync(mainExe).size)})`,
);

// ---- 2. 编译"纯壳"卸载器（第一次 cargo build）----
// 图标/向导 logo 由第 1 步 beforeBuildCommand 的 gen-icons.mjs 同步；
// 先删残留 zip：防止上次打包的 zip 被嵌进纯壳，导致壳内嵌套旧安装包；
// 前端 build 必须在 cargo build 之前（壳同样嵌 frontendDist，卸载向导 UI 依赖它）
step("build pure shell wizard (vite + 1st cargo build)");
rmSync(bundleZip, { force: true });
if (!existsSync(join(installerDir, "node_modules"))) {
  run("npm install", installerDir);
}
run("npm run build", installerDir);
run(CARGO_BUILD_CMD, installerCrateDir);
if (!existsSync(wizardExe)) {
  fail(`shell wizard binary not found: ${wizardExe}`);
}
const shellSize = statSync(wizardExe).size;
console.log(`[build-installer] shell wizard: ${fmtMB(shellSize)}`);

// ---- 3. 打包 app-bundle.zip（zip 根须为 {mainBin}.exe + {wizardBin}.exe，
// 与 config.rs 的 MAIN_EXE_NAME / WIZARD_EXE_NAME 约定一致）----
step("pack app-bundle.zip (main exe + shell wizard)");
const staging = mkdtempSync(join(tmpdir(), "app-bundle-"));
try {
  copyFileSync(mainExe, join(staging, `${mainBin}.exe`));
  copyFileSync(wizardExe, join(staging, `${wizardBin}.exe`));
  // Compress-Archive 为 deflate 压缩，与安装器 zip crate 的 deflate feature 兼容；
  // zip 根仅平铺文件，无路径分隔符兼容性问题
  run(
    `powershell -NoProfile -Command "Compress-Archive -Force -Path '${staging}\\*' -DestinationPath '${bundleZip}'"`,
  );
} finally {
  rmSync(staging, { recursive: true, force: true });
}
if (!existsSync(bundleZip) || statSync(bundleZip).size === 0) {
  fail("app-bundle.zip missing or empty after Compress-Archive");
}
const zipSize = statSync(bundleZip).size;
console.log(`[build-installer] bundle: ${bundleZip} (${fmtMB(zipSize)})`);

// ---- 4. 编译完整安装器（第二次 cargo build，嵌入 zip，原地覆盖纯壳产物）----
step("build full installer (2nd cargo build, embeds zip)");
run(CARGO_BUILD_CMD, installerCrateDir);
if (!existsSync(wizardExe)) {
  fail(`installer binary not found: ${wizardExe}`);
}
const wizardSize = statSync(wizardExe).size;
console.log(
  `[build-installer] full installer: ${fmtMB(wizardSize)} ` +
    `(shell ${fmtMB(shellSize)} + bundle ${fmtMB(zipSize)})`,
);
// 嵌入校验：完整安装器体积必须大于 zip（防零字节占位），
// 且明显大于纯壳（差值 ≈ zip 体积，防第二次编译未重新嵌入）
if (wizardSize <= zipSize) {
  fail(
    `installer size (${fmtMB(wizardSize)}) <= bundle size (${fmtMB(zipSize)}), ` +
      "embedded bundle looks like the zero-byte placeholder, check build.rs",
  );
}
if (wizardSize <= shellSize) {
  fail(
    `installer size (${fmtMB(wizardSize)}) <= shell size (${fmtMB(shellSize)}), ` +
      "2nd cargo build did not re-embed the bundle zip",
  );
}

// ---- 5. 收集产物 ----
step("collect artifact");
const mainConf = JSON.parse(
  readFileSync(join(root, "src-tauri", "tauri.conf.json"), "utf-8"),
);
const outDir = join(root, "dist-installer");
// 先清空旧产物，避免新旧命名并存
rmSync(outDir, { recursive: true, force: true });
mkdirSync(outDir, { recursive: true });
// Tauri 官方 NSIS 命名风格；文件名含 setup 不触发 UAC 启发式检测
// （仅对未嵌入 manifest 的 32 位进程生效，本产物为 64 位 + asInvoker manifest）
const output = join(
  outDir,
  `${mainConf.productName}_${mainConf.version}_x64-setup.exe`,
);
copyFileSync(wizardExe, output);
console.log(
  `[build-installer] done in ${elapsed()}: ${output} (${fmtMB(wizardSize)})`,
);
