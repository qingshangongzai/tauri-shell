# 轻壳 — Tauri 桌面应用模板

一个极简的 Tauri v2 启动模板——将任何 HTML 页面打包成 Windows 桌面程序。Rust 层零业务逻辑，纯空壳，故名「轻壳」。

> 🏠 [Gitee 仓库](https://gitee.com/qingshangongzai/tauri-shell/) · 开发者 [青山公仔](https://gitee.com/qingshangongzai)

**特点：**
- 无边框窗口 + 仿原生标题栏（最小化 / 最大化 / 关闭）
- 内置丰富的示例界面：侧边栏导航、仪表盘、工具页、设置页（含深色模式）、关于页
- 自带安装/卸载向导（Tauri 2 + React 自绘 UI，替代 NSIS）：双路径安装、按需 UAC 提权、控制面板集成
- 右键菜单禁用、文本选定禁用（按需移除）
- 构建时自动压缩 HTML，减小体积
- 图标仅维护一张源图，其余由构建脚本自动生成

## 目录结构

```
轻壳/
├── dist/
│   └── index.html          ← 你的网页放这里
├── src-tauri/
│   ├── src/
│   │   ├── main.rs         ← 入口（无需修改）
│   │   └── lib.rs          ← Tauri 空壳（无需修改）
│   ├── capabilities/
│   │   └── default.json    ← 窗口权限（无需修改）
│   ├── icons/
│   │   └── logo.svg        ← 只维护这一张源图（正方形、透明背景 SVG）
│   ├── Cargo.toml          ← Rust 项目配置
│   ├── build.rs            ← 构建脚本（无需修改）
│   └── tauri.conf.json     ← 窗口大小 / 标题等
├── installer/              ← 安装/卸载向导子包（Tauri 2 + React）
│   ├── src/
│   │   └── product.ts      ← 安装器前端文案配置（改名时修改）
│   └── src-tauri/
│       └── src/config.rs   ← 安装器产品常量（改名时修改）
├── docs/
│   └── 安装器说明.md       ← 安装器详细说明
├── scripts/
│   ├── gen-icons.mjs       ← 图标生成（构建时自动调用，只产出 Windows 所需）
│   └── build-installer.mjs ← 安装器打包链（npm run build 调用）
├── package.json            ← Node 脚本
├── minify.cjs              ← HTML 压缩（构建时自动调用）
├── sync-version.cjs        ← 版本号同步（构建时自动调用）
└── .gitignore
```

## 前提条件

1. 安装 [Rust](https://rustup.rs/)
2. 安装 [Node.js](https://nodejs.org/)（≥ 18）
3. 安装 npm 依赖：

```bash
npm install
```

> 安装器子包的依赖在首次打包时由构建脚本自动安装，也可手动执行 `npm --prefix installer install`。

## 快速开始

### 1. 编写你的网页

直接修改 `dist/index.html` 即可。

模板已包含一个**功能丰富的示例界面**，展示 Tauri 桌面应用可以做什么：

- **仪表盘** — 欢迎卡片、技术栈统计、快速开始指南
- **工具页** — 计数器、剪贴板读写、秒表/倒计时、便签本、JSON 格式化、取色器、Base64 编解码、摩斯编码
- **能力展示** — 快捷键面板、拖放区域、应用内通知（Toast）
- **设置页** — 深色 / 浅色主题切换、表单控件、本地存储持久化
- **关于页** — 应用图标、技术栈信息、许可证

如果你完全用自己的 HTML，可选择：

- **保留自定义标题栏**：`<div class="titlebar">` 及其子元素，以及尾部 `<script type="module">` 中的窗口控制逻辑。这是无边框窗口的拖拽和关闭 / 最小化 / 最大化功能。
- **使用系统原生标题栏**：在 `tauri.conf.json` 中将 `"decorations"` 改为 `true`，然后删除 HTML 中的标题栏代码。

### 2. 预览（实时刷新）

```bash
npm run tauri dev
```

修改 `dist/index.html`，窗口会自动刷新。

### 3. 构建安装包 EXE

```bash
npm run build
```

执行自带的安装器打包链：主应用构建 → 安装向导编译 → 主程序嵌入，
产物为 `dist-installer/{产品名}_{版本}_x64-setup.exe`——单文件分发，
双击即进入安装向导（欢迎 → 路径 → 选项 → 进度 → 完成），卸载走控制面板。
详细原理与自定义见 [docs/安装器说明.md](docs/安装器说明.md)。

> 首次打包需完整编译安装器依赖，耗时较长；后续增量构建会快很多。
> 只改了安装器、主应用未变时，可用 `npm run build -- --skip-main` 跳过主应用构建。

**备用方案：NSIS 安装包**（Tauri 官方打包，速度快但无自定义向导 UI）：

```bash
npm run build:nsis
```

产物在 `src-tauri/target/release/bundle/nsis/` 目录下。

仅需裸 EXE（不要安装器）时：

```bash
npm run tauri build -- --no-bundle
```

构建产物在 `src-tauri/target/release/` 目录下。

> 构建时会自动执行：版本号同步 → 图标生成（从 `logo.svg`） → HTML 压缩，一步到位。
> 安装向导的图标、页面 logo 与版本号同样源自主项目，无需单独维护。

## 自定义配置

需要修改的地方（按优先级）：

### 必改项

| 文件 | 字段 | 说明 |
|------|------|------|
| `tauri.conf.json` | `productName` | 应用名称 |
| `tauri.conf.json` | `identifier` | 唯一标识，建议 `com.xxx.yyy` |
| `tauri.conf.json` | `windows[0].title` | 窗口标题 |
| `tauri.conf.json` | `windows[0].width/height` | 窗口尺寸 |
| `Cargo.toml` | `name` | 内部名称（需与 `bin.name`、`lib.name` 一致） |
| `Cargo.toml` | `[package.metadata.tauri-winres]` 下所有字段 | Windows EXE 属性（右键文件 → 属性 → 详细信息） |
| `package.json` | `name` | npm 包名 |
| `dist/index.html` | `<title>`、`.titlebar-title` | 页面标题、标题栏文字 |
| `installer/src-tauri/src/config.rs` | 全部常量 | 安装器产品信息（显示名、exe 名、注册表键等，文件内有逐项说明） |
| `installer/src/product.ts` | 全部常量 | 安装器前端文案（与 `config.rs` 保持一致） |
| `installer/src-tauri/tauri.conf.json` | `productName` / `identifier` / 窗口 `title` | 安装器窗口信息 |
| `installer/src-tauri/Cargo.toml` | `[[bin]] name`（需等于 config.rs 的 `WIZARD_EXE_NAME` 去掉 `.exe`）及 `tauri-winres` 字段 | 安装器 EXE 名与属性 |
| `installer/index.html` | `<title>` | 安装器页面标题 |

### 可选改动

| 文件 | 字段 | 说明 |
|------|------|------|
| `tauri.conf.json` | `version` | 版本号（`sync-version.cjs` 会自动同步到其他文件） |
| `tauri.conf.json` | `windows[0].resizable` | 是否可调整大小 |
| `tauri.conf.json` | `windows[0].decorations` | `false`= 无边框 + 自定义标题栏，`true`= 系统原生标题栏 |
| `src-tauri/icons/logo.svg` | — | 替换为你的应用图标（正方形、透明背景 SVG），构建时自动生成 EXE 图标并同步给安装器 |

### 版本号规则

修改版本号**只需改一处**：`tauri.conf.json` 中的 `version` 字段。

构建时 `sync-version.cjs` 会自动将版本号同步到 `package.json`、`Cargo.toml`
以及安装器子包（`installer/package.json`、`installer/src-tauri/Cargo.toml`、
`installer/src-tauri/tauri.conf.json`）。

## 隐藏 Web 特征

模板已内置以下措施防止暴露网页本质：

- `<body oncontextmenu="return false;">` — 禁用右键菜单
- `body { user-select: none }` — 禁用文本选定

如需启用，删除对应代码即可。

## 窗口显示白屏？

这是设计如此——`tauri.conf.json` 中 `"visible": false`，页面加载完成后 JS 调用 `plugin:window|show` 显示窗口，避免白屏闪烁。

## 许可证

本项目采用 [木兰宽松许可证 第 2 版（MulanPSL-2.0）](http://license.coscl.org.cn/MulanPSL2)。
