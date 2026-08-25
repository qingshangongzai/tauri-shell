# 轻壳（Tauri Shell）— AI 协作入口

把任意单个 HTML 页面打包成 Windows 桌面应用的极简 Tauri v2 壳模板：前端为原生单文件 HTML/CSS/JS（`dist/index.html`，零依赖、资源全内联），Rust 层几乎零逻辑，另含安装/卸载向导子包（`installer/`，Tauri 2 + React）。

## 规则（动手前先读）

- **编码规则唯一真源**：`开发规范.md` —— 开始任何编码 / 审查 / 提交 / 写方案任务前，先读它；任何 skill 的规则与它冲突时，以它为准。

## Skill 触发对照（用户说关键词时自动用）

| 用户意图 | Skill | 触发词 |
|---|---|---|
| 审查代码 | `code-review` | 检查代码、代码审查、review、核对修改 |
| 生成提交信息 | `commit-message-generator` | 提交信息、commit、合并分支、merge |
| 创建分支 | `create-branch` | 创建分支、开个分支、分支名 |
| 写/更新方案文档 | `plan-doc` | 写方案、实施计划、落地记录、写思路 |
| 只分析不修改 | `question-mode` | 提问、提问模式 |
| UI 动效设计/审查 | `ui-animation` | 加动画、让这更流畅、审查动画、滑动手势 |

## 约定

- skill 以 `skills/`（已提交仓库）为唯一维护源；各工具目录（`.qoder/`、`.trae/`、`.codebuddy/`、`.opencode/`、`.zcode/`）下的副本从它同步，不直接修改副本。
- 修改/新增 skill 只改 `skills/<name>/SKILL.md` 再同步到各工具目录（各工具格式基本兼容：`SKILL.md` + frontmatter 的 `name`/`description`）；新增 skill 在下方触发表补一行。
- 版本号唯一来源：`src-tauri/tauri.conf.json` 的 `version` 字段（构建时 `sync-version.cjs` 自动同步，禁止手改任何一处）。
- 依赖版本唯一来源：`package.json` / `installer/package.json` / 两处 `Cargo.toml`。
- 设计语言唯一来源：`docs/“去线留白”设计语言.md`；页面硬性要求见 `docs/AI生成页面需求模板.md`。
- 托盘契约（`dist/index.html` 的 `data-tauri-tray` 标记 / `build.rs` / `src-tauri/src/lib.rs` cfg）三处必须一致，改动后跑 `cargo test` 与 `npm run test:tray`。
