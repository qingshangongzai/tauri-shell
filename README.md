# 轻壳 — Tauri 桌面应用模板

一个极简的 Tauri v2 启动模板——将任何 HTML 页面打包成 Windows 桌面程序。Rust 层几乎零逻辑（仅拖放示例的文件大小命令与系统托盘两处能力，均可按需移除），故名「轻壳」。

除了作为壳模板，示例页内置的 Toast / Modal / Tab / Tooltip / 右键菜单等组件均遵循统一的「去线留白」设计语言（见 [docs/“去线留白”设计语言.md](docs/“去线留白”设计语言.md)），纯 CSS + 原生 JS、零依赖，也可当作**轻量参考组件库**按需拆用——独立拆分版见 [components/](components/) 目录。

> 🏠 [Gitee 仓库](https://gitee.com/qingshangongzai/tauri-shell/) · 开发者 [青山公仔](https://gitee.com/qingshangongzai)

**特点：**
- 无边框窗口 + 仿原生标题栏（最小化 / 最大化 / 关闭）
- 系统托盘：关闭窗口最小化到托盘（设置页可关），托盘右键为「去线留白」自绘菜单，可按需移除
- 内置丰富的示例界面：侧边栏导航、仪表盘、工具页、能力展示、组件页、设置页（含深色模式）、关于页
- 可拆用的内置组件：Toast 通知、Modal 弹窗、Tab 标签页、Progress 进度条、Badge 徽章、Button 按钮、Input 输入框、Switch 开关、Select 下拉选择器、Tooltip、自定义右键菜单
- 示例界面三语切换（简体中文 / English / 日本語），零依赖内联字典，切换即时生效
- 自带安装/卸载向导（Tauri 2 + React 自绘 UI，替代 NSIS）：双路径安装、按需 UAC 提权、控制面板集成
- 右键菜单禁用、文本选定禁用（按需移除）
- 构建时自动压缩 HTML，减小体积
- 图标仅维护一张源图，其余由构建脚本自动生成

## 两种使用方式

**① 纯壳 — 打包你自己的 HTML**

把你的网页放进 `dist/index.html`，一键打包为 Windows 桌面程序，示例界面随之被替换。不关心示例页长什么样，直接看下方「快速开始」。

**② 同风格开发 — 复用组件与样式**

认可示例页的「去线留白」风格，可以从 `components/` 目录起步：`tokens.css`（设计令牌）、`starter.html`（风格骨架模板，替换 `dist/index.html` 即得同风格空应用）、以及 Toast / Modal / Tab / Progress / Badge / Button / Input / Switch / Select / Tooltip / 右键菜单十一个自包含 demo（双击浏览器即可预览，复制三段注释标出的 CSS/HTML/JS 即可移植）。

两条路径的详细步骤（标题栏取舍、改名清单、设计令牌速查、组件复制指引等）见 [docs/使用说明.md](docs/使用说明.md)。

## 目录结构

```
轻壳/
├── dist/
│   ├── index.html          ← 你的网页放这里
│   ├── tray-menu.html      ← 托盘右键菜单页（不需托盘可删，见 docs/托盘说明.md）
│   └── serve.json          ← dev 静态服务器配置（勿删，否则 dev 下托盘菜单页会被重写成主页）
├── src-tauri/
│   ├── src/
│   │   ├── main.rs         ← 入口（无需修改）
│   │   └── lib.rs          ← Tauri 轻壳（拖放示例的文件大小命令 + 托盘，均可按需移除）
│   ├── capabilities/
│   │   ├── default.json    ← 主窗口权限（无需修改）
│   │   └── tray-menu.json  ← 托盘菜单窗口权限（无需修改）
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
├── components/             ← 参考组件库（不进构建产物，dist/index.html 为权威源）
│   ├── tokens.css          ← 「去线留白」设计令牌
│   ├── starter.html        ← 风格骨架模板（替换 dist/index.html 即得同风格空应用）
│   └── *.html              ← Toast / Modal / Tab / Progress / Badge / Button / Input / Switch / Select / Tooltip / 右键菜单 demo
├── docs/
│   ├── 使用说明.md         ← 两条使用路径详解（改名清单、组件复制指引）
│   ├── 安装器说明.md       ← 安装器详细说明
│   └── 托盘说明.md         ← 托盘原理、定制与移除步骤
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

- **仪表盘** — 欢迎卡片、实时时钟、系统信息、快捷操作、活动日志
- **工具页** — 计数器、剪贴板读写、秒表/倒计时、便签本、JSON 格式化、取色器、Base64 编解码、摩斯编码
- **能力展示** — 快捷键面板、拖放区域
- **组件页** — Toast / Modal / Tab / Progress / Badge / Button / Input / Switch / Select / Tooltip 等组件的活体示例，样式与逻辑均可直接复制到自己的项目
- **设置页** — 深色 / 浅色主题切换、语言切换（简体中文 / English / 日本語）、关闭时最小化到托盘开关、设置导入/导出、本地存储持久化
- **关于页** — 应用图标、技术栈信息、许可证

交互细节也已就位：侧边栏折叠（`Ctrl+B`）、键盘全局导航（`Ctrl+1~6`）、自定义右键菜单、页面状态保持等。

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

改成你自己的应用需要动哪些文件（`productName`、`identifier`、EXE 属性、安装器常量等），
见 [docs/使用说明.md](docs/使用说明.md) 中的「改名清单」（必改项 / 可选改动两张表，逐项说明）。

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

## 按需移除托盘

模板默认启用系统托盘：关闭窗口最小化到托盘（设置页「窗口」卡片可关，关闭后点 X 直接退出），
托盘左键还原主窗口，右键弹出与应用同风格的自绘菜单（透明 Webview 小窗渲染，非原生菜单）。
与 get_file_size 同为少数 Rust 侧能力。

工作原理、菜单项定制、注意事项与完整移除步骤见 [docs/托盘说明.md](docs/托盘说明.md)。

## 窗口显示白屏？

这是设计如此——`tauri.conf.json` 中 `"visible": false`，页面加载完成后 JS 调用 `plugin:window|show` 显示窗口，避免白屏闪烁。

## 许可证

本项目采用 [木兰宽松许可证 第 2 版（MulanPSL-2.0）](http://license.coscl.org.cn/MulanPSL2)。
