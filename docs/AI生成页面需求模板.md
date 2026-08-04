# AI 生成页面需求模板

> **使用方法**
> 把整个文档发给任意在线 AI 助手，并附上一句你的需求即可，例如：
> 「请根据以下文档，给我生成一个**食品保质期计算器**的页面」
> AI 会返回一个完整的 HTML 页面文件——复制保存后替换 `dist/index.html` 即可打包成桌面应用。
>
> **维护注记**：本文档的规范部分与 `docs/使用说明.md`（标题栏章节、右键菜单章节）、`docs/"去线留白"设计语言.md`、`components/starter.html` 同源，改动任一来源请同步本模板。

---

## 给你的任务（AI 阅读）

根据用户（在你对话中）提出的功能需求，生成一个完整的 HTML 页面文件。

- 使用原生 HTML / CSS / JavaScript 实现，不引入任何前端框架。
- 所有样式与脚本必须内联在文件内部；**不得引用任何外部资源**（外链 CSS、CDN、外部图片、外部字体、网络图标库）。
- 页面必须满足「一、页面结构规范」与「二、设计规范」的全部要求。
- 用户需求未明确的部分，采用本文档各节的**默认选项**。
- 「三、起步骨架」提供了一份合规骨架：**在骨架上填充你的内容，保持骨架的标题栏、样式与脚本不变**（用户要求原生标题栏时除外，见 1.2 方式 B）。
- 输出前逐条通过「四、交付前自查清单」，输出时直接给出完整可用的文件内容。

---

## 一、页面结构规范

### 1.1 单文件与资源内联（必须）

- 页面必须是**单个 HTML 文件**：CSS 写在 `<style>` 内，JS 写在 `<script>` 内，图标用内联 SVG。
- 禁止外链任何资源；字体使用系统字体栈（见令牌 `--font`），不引入网络字体。
- 小尺寸图片用内联 `<svg>` 或 base64 数据；不要引用任何外部文件路径。
- 需要持久化的数据（设置、记录、历史）用 `localStorage` 保存，重新打开页面后自动恢复。

### 1.2 窗口形态（二选一，默认方式 A）

**方式 A：自绘标题栏（无边框窗口）——默认**

页面自带一条位于顶部的自绘标题栏，与页面设计融为一体：

1. **标题栏元素**：位于 `<body>` 最前面，固定定位，高度 40px（取令牌 `--titlebar-h`），并带有属性 `data-tauri-drag-region`——这是窗口拖拽标记，**必须原样保留，禁止改名或删除**。
2. **结构**：左侧应用标题文字，右侧三个窗口按钮（最小化 / 最大化 / 关闭）。
3. **三个按钮必须接到窗口命令**：脚本见下方，命令字符串**原样使用，不要臆造或修改**。
4. **标题文字**加 `pointer-events: none`，保证按住文字也能拖动窗口。
5. **双击标题栏 = 最大化/还原**：内置行为，无需编写代码。
6. **浏览器预览降级**：在普通浏览器中打开时没有窗口可控制，脚本应自动隐藏标题栏。

标题栏 HTML 结构（按钮 id 是脚本依赖，必须保留）：

```html
<div class="titlebar" data-tauri-drag-region>
    <span class="titlebar-title">（应用名称，与 <title> 保持一致）</span>
    <div class="window-controls">
        <button class="win-btn win-btn-min" id="winMin" title="最小化" aria-label="最小化"></button>
        <button class="win-btn win-btn-max" id="winMax" title="最大化" aria-label="最大化"></button>
        <button class="win-btn win-btn-close" id="winClose" title="关闭" aria-label="关闭"></button>
    </div>
</div>
```

窗口控制脚本（放在 `<body>` 末尾的 `<script>` 内，原样使用）：

```html
<script>
{
    const titlebar = document.querySelector('.titlebar');
    const internals = window.__TAURI_INTERNALS__;

    if (!internals) {
        // 普通浏览器预览：没有窗口可控制，隐藏标题栏
        if (titlebar) titlebar.style.display = 'none';
        const content = document.querySelector('.content');
        if (content) { content.style.marginTop = '0'; content.style.height = '100vh'; }
    } else {
        const invoke = cmd => internals.invoke(cmd);

        document.getElementById('winMin').addEventListener('click', () => invoke('plugin:window|minimize'));
        document.getElementById('winMax').addEventListener('click', () => invoke('plugin:window|toggle_maximize'));
        // 关闭按钮：默认直接关闭；用户明确要求"最小化到托盘"时按 1.5 节替换
        document.getElementById('winClose').addEventListener('click', () => invoke('plugin:window|close'));

        // 最大化/还原图标切换
        const maxBtn = document.getElementById('winMax');
        const updateMaxBtn = async () => {
            try {
                const maximized = await invoke('plugin:window|is_maximized');
                maxBtn.classList.toggle('maximized', maximized);
                maxBtn.title = maximized ? '还原' : '最大化';
            } catch { /* 忽略 */ }
        };
        window.addEventListener('resize', updateMaxBtn);
        updateMaxBtn();
    }
}
</script>
```

> `window.__TAURI_INTERNALS__` 与 `plugin:window|*` 命令字符串是环境提供的固定接口，照抄即可；在普通浏览器中运行时 `internals` 不存在，脚本自动走降级分支。

**方式 B：系统原生标题栏——当用户明确要求时**

用户说"不要自绘标题栏"「用系统标题栏」「简单一点就行」时采用：**页面不包含任何标题栏代码**，窗口自动使用系统原生标题栏（最小化/最大化/关闭由系统提供），页面无需编写任何窗口控制脚本。

实现方法：在起步骨架基础上做三处删除——
1. 删除标题栏 HTML 元素（`.titlebar` 块）；
2. 删除标题栏相关 CSS（`.titlebar` 至 `.win-btn.maximized::before` 整段）与 `--close-hover` / `--close-active` 令牌；
3. 删除窗口控制脚本（整个 `if (!internals) ... else { ... }` 块），并把 `.content` 改为 `height: 100vh; margin-top: 0`。

### 1.3 内容滚动规则（必须）

- `body` 自身**不滚动**（`overflow: hidden`）；滚动交给内容容器 `.content`（`overflow-y: auto`）。
- 布局：标题栏 `position: fixed` 置顶（方式 A）；`.content` 高度 `calc(100vh - var(--titlebar-h))`、`margin-top: var(--titlebar-h)`（方式 B 为 `100vh` / `0`）。
- 原因：若让 body 滚动，滚动条会贯穿整个视口、从标题栏右侧一路延伸到底部——不符合本模板规范。
- 滚动条美化（可选）：用 `::-webkit-scrollbar` 做细窄圆角样式，颜色取令牌 `--divider`（示例见骨架）。

### 1.4 深色模式（默认不提供）

- 默认**不提供**深色模式：页面固定使用浅色主题（骨架保留深色令牌块但不启用，无切换入口）。
- 用户明确要求"支持深色模式"「要深色/暗色主题」时，才提供手动切换：
  1. 启用骨架中的 `[data-theme="dark"]` 令牌块（骨架默认保留，无需改动）；
  2. 在页面放置切换控件（设置区开关或页面角落按钮均可），并加入以下主题切换脚本：

```html
<script>
{
    const savedTheme = localStorage.getItem('tauri-shell-theme');
    if (savedTheme === 'dark') document.documentElement.setAttribute('data-theme', 'dark');

    function applyTheme(isDark) {
        const apply = () => document.documentElement.setAttribute('data-theme', isDark ? 'dark' : '');
        if (document.startViewTransition && !matchMedia('(prefers-reduced-motion: reduce)').matches) {
            document.startViewTransition(apply);
        } else { apply(); }
    }
    // 切换控件 change 时调用：
    // applyTheme(checked);
    // localStorage.setItem('tauri-shell-theme', checked ? 'dark' : 'light');
    // 控件初始 checked 状态与 localStorage 保持一致
}
</script>
```

- 机制：`<html>` 上的 `data-theme="dark"` 属性触发令牌整体反转。

### 1.5 最小化到托盘（默认不启用）

用户明确要求"最小化到托盘"「关闭后后台运行」时才启用，需做**两处修改**：

1. 在 `<html>` 标签上添加属性 `data-tauri-tray`（原样保留）；
2. 窗口控制脚本中的关闭按钮改为：

```js
// 关闭 = 隐藏到系统托盘，应用保持后台运行（点击托盘图标恢复窗口）
document.getElementById('winClose').addEventListener('click', () => invoke('plugin:window|hide'));
```

未要求时保持 1.2 的 `plugin:window|close`，**不要**添加 `data-tauri-tray`。

### 1.6 右键菜单（默认屏蔽默认菜单）

默认加入以下脚本，屏蔽浏览器默认右键菜单（前进/后退、刷新、查看源代码等），使应用不暴露网页特征：

```html
<script>
    // 屏蔽默认右键菜单（浏览器预览同样生效）
    document.addEventListener('contextmenu', e => e.preventDefault());
</script>
```

用户要求"右键菜单要应用风格的自绘菜单"时：在屏蔽默认菜单的同时，右键在光标位置弹出应用风格的自绘菜单。**注意**：屏蔽默认菜单后，输入框的"粘贴/剪切"等默认项也会消失，自绘菜单需提供对应动作（用 `navigator.clipboard` 读写剪贴板）。完整实现：

```html
<style>
    .context-menu {
        position: fixed;
        z-index: 9999;
        min-width: 150px;
        padding: 4px;
        border-radius: var(--radius-sm);
        background: var(--card);
        box-shadow: var(--shadow-flyout);
        font-size: 0.78rem;
        color: var(--text-primary);
        display: none;
    }
    .context-menu.visible { display: block; }
    .context-menu-item {
        display: flex;
        align-items: center;
        gap: 8px;
        padding: 6px 10px;
        border-radius: 4px;
        cursor: pointer;
        white-space: nowrap;
    }
    .context-menu-item:hover { background: var(--hover); }
    .context-menu-item.disabled { color: var(--text-placeholder); pointer-events: none; }
    .context-menu-divider { height: 1px; margin: 4px 8px; background: var(--divider); opacity: 0.6; }
</style>

<div class="context-menu" id="contextMenu">
    <div class="context-menu-item" data-action="copy">复制</div>
    <div class="context-menu-item" data-action="paste">粘贴</div>
    <div class="context-menu-divider"></div>
    <div class="context-menu-item" data-action="reload">刷新</div>
</div>

<script>
{
    const menu = document.getElementById('contextMenu');
    const items = [...menu.querySelectorAll('.context-menu-item')];
    const hide = () => menu.classList.remove('visible');

    // 右键：屏蔽默认菜单，并在光标位置显示自绘菜单
    document.addEventListener('contextmenu', e => {
        e.preventDefault();
        const editable = !!e.target.closest('input, textarea, [contenteditable]');
        items.forEach(i => {
            const needsEditable = (i.dataset.action === 'copy' || i.dataset.action === 'paste');
            i.classList.toggle('disabled', needsEditable && !editable);
        });
        menu.style.left = Math.min(e.clientX, innerWidth - 170) + 'px';
        menu.style.top = Math.min(e.clientY, innerHeight - 130) + 'px';
        menu.classList.add('visible');
    });
    // 外点 / ESC 关闭
    document.addEventListener('click', hide);
    document.addEventListener('keydown', e => { if (e.key === 'Escape') hide(); });

    // 菜单项动作（菜单项可按需增删）
    items.forEach(item => item.addEventListener('click', async () => {
        const action = item.dataset.action;
        if (action === 'copy') {
            const input = document.activeElement;
            if (input && input.value !== undefined) await navigator.clipboard.writeText(input.value);
        }
        if (action === 'paste') {
            const input = document.activeElement;
            if (input && input.value !== undefined) input.value = await navigator.clipboard.readText();
        }
        if (action === 'reload') location.reload();
        hide();
    }));
}
</script>
```

> 菜单项与动作按用户需求增删即可；样式一律使用令牌，保持与页面一致。

### 1.7 屏蔽浏览器快捷键（默认加入，可选）

以下代码让**打包后的应用**屏蔽 F5 / F12 / Ctrl+R / Ctrl+U 等浏览器快捷键，并在浏览器预览时自动不生效（保留调试键）：

```html
<script>
// 仅在打包版环境中生效，浏览器预览保留 F5/F12 便于调试
if (location.protocol === 'tauri:' || location.hostname === 'tauri.localhost') {
    const BLOCKED_CTRL_KEYS = ['r', 'p', 'u', 'f', 'g', 'j', '+', '-', '=', '0'];
    document.addEventListener('keydown', e => {
        const key = e.key.toLowerCase();
        if (
            key === 'f5' || key === 'f3' || key === 'f12' ||
            (e.ctrlKey && e.shiftKey && ['i', 'j', 'c'].includes(key)) ||
            (e.ctrlKey && !e.altKey && BLOCKED_CTRL_KEYS.includes(key)) ||
            (e.altKey && (key === 'arrowleft' || key === 'arrowright'))
        ) { e.preventDefault(); }
    }, { capture: true });
    // Ctrl+滚轮页面缩放
    document.addEventListener('wheel', e => { if (e.ctrlKey) e.preventDefault(); }, { passive: false, capture: true });
    // 鼠标侧键历史前进/后退
    document.addEventListener('mouseup', e => { if (e.button === 3 || e.button === 4) e.preventDefault(); });
}
</script>
```

### 1.8 图标（按需）

- 需要图标时使用**内联 SVG 线条图标**（`fill="none" stroke="currentColor"`，Lucide 风格），颜色随文字令牌自动适配。
- 禁止使用 emoji 代替图标、禁止引入外部图标库。
- 常用图标可定义在 `<body>` 开头的隐藏雪碧图中，页面内用 `<use href="#图标id">` 引用：

```html
<svg xmlns="http://www.w3.org/2000/svg" style="display:none" aria-hidden="true">
    <symbol id="i-check" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M20 6 9 17l-5-5"/></symbol>
    <symbol id="i-x" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M18 6 6 18"/><path d="m6 6 12 12"/></symbol>
    <symbol id="i-info" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><circle cx="12" cy="12" r="10"/><path d="M12 16v-4"/><path d="M12 8h.01"/></symbol>
</svg>
<!-- 使用：<svg class="icon" aria-hidden="true" style="width:16px;height:16px"><use href="#i-check"></use></svg> -->
```

- 图标尺寸 16–20px，与文字垂直对齐；装饰性图标加 `aria-hidden="true"`。

### 1.9 多功能 / 多页面组织（按需）

用户需求包含多个功能或页面时：

- 推荐**左侧导航栏 + 页面切换**（功能多），或**顶部 Tab 切换**（功能少、简单场景）。
- 导航栏样式对齐设计语言：选中态用背景 `--accent-bg` + 文字 `--accent` 加粗，悬停用 `--hover`，不用下划线和边框（锚点导航的隐形表达）。
- 导航项与页面用按钮 + `data-page` / `section#page-xxx` 的简单 JS 切换即可，无需框架。
- 单个功能的简单应用无需导航，直接在 `.content` 里铺内容。

---

## 二、设计规范：「去线留白」

### 2.1 执行铁律（违反任何一条即不合格）

1. **间距**：所有间距必须是 8px 的整数倍——同一信息组内 8/16px，不同信息组之间 24/32px，页面章节/模块之间 48/64px，页面边距 16/24px。
2. **无边框原则**：表达分隔与层级用**留白、色块、弥散阴影**，不用边框线；仅高密度数据界面（表格类）与警示场景可保留分割线。
3. **令牌取色**：颜色一律从令牌取值，**禁止写死色值**；需要新颜色时先定义成新令牌再使用。
4. **文字层级**：用不透明度阶梯表达——主标题 `--text-primary`（87%）、正文 `--text-body`（78%）、辅助说明 `--text-auxiliary`（52%）、占位符 `--text-placeholder`（30%），不用灰阶。
5. **动效**：时长只用 `--duration-fast`（100ms）/ `--duration-normal`（200ms）/ `--duration-slow`（300ms），缓动统一 `var(--ease)`；微交互 100–150ms，页面转场 250–350ms；必须带 `prefers-reduced-motion` 降级。

### 2.2 设计语言

**核心主旨：** 舒适自然、内容优先、视觉降噪、认知减负。

**设计宣言：** 尽量减少不必要的分割线，而非一刀切地全部去掉；让界面更加轻盈，更易聚焦内容；用空间、色块与透明度来表达层级，而非依赖生硬的线条。

#### 一、设计理念概述

"去线留白"以"减法"为核心：剥离非必要装饰，以负空间凸显内容，用更自然的方式构建秩序——线条是强加于内容的框，而留白与色块则是内容生长出的呼吸。其设计依据：每条分割线、每个边框都会增加认知负荷（视觉熵），减少它们就是把用户的注意力集中在内容本身。

#### 二、视觉执行系统：以空间构建秩序

**1. 去线化：以空间代替边界**

- **无边界列表**：尽量减少传统的像素级灰色分割线，作为默认方向。利用人眼的"格式塔"补全效应，通过间距暗示边界。交互状态优先使用背景色微变（如透明度 5% 的黑色遮罩）或弥散阴影加深来反馈，而非边框高亮。
- **隐形卡片化**：减少清晰的线框使用，优先采用极微弱的弥散阴影（Y 轴偏移 2px，模糊 8px，透明度 5%–8%）或极浅的背景色差（灰度阶差 ≤ 3%）。
- **表单控件重构**：输入框尽量采用"填充式"设计取代下划线或包围式边框——输入框拥有极浅（1%-2% 灰度）的独立背景色，聚焦时背景加深或光标变色。标签与输入域之间优先依靠垂直对齐与字重对比建立关联，而非线条连接。

**2. 留白艺术：数学化的呼吸感**

留白不是随意留，而是基于一套严格的间距标尺。以 **8px** 为基准单元，所有间距均为基准单元的整数倍：

| 场景 | 间距值 | 说明 |
|:---|:---|:---|
| 同一信息组内 | 8px / 16px | 紧密关联，如图标与文字、标签组、按钮与图标 |
| 不同信息组之间 | 24px / 32px | 明确区隔，如表单字段组、卡片内不同模块 |
| 页面章节 / 模块间距 | 48px / 64px | 宏观结构划分，如卡片与卡片之间 |
| 页面边距（安全区） | 16px / 24px | 视口边缘留白，桌面端 24px+ |

**3. 层级重构：排版与色阶**

- **字重与字号对比**：主标题用粗体深色，辅助信息用细体浅灰，通过强烈反差区分主次。
- **不透明度阶梯**：通过同一色彩的不同不透明度表达信息层级（数值见铁律 4，具体令牌见 2.3）。
- **背景色阶**：利用极其微弱的灰度阶差区分背景层与内容层（阶差 ≤ 3%），实现"润物细无声"的层级划分。

**4. 沉浸式色彩体系**

- **克制的色彩数量**：严格限制为三类——一个品牌主色、一套低饱和度功能辅助色、一套完善的中性色系。
- **纯粹暗黑**：深色模式不使用纯黑（#000000），而是采用深灰（如 #202020）、藏蓝等低饱和度色彩；无刺眼分割线的设计，使暗光视觉体验极其柔和。

**5. 响应式留白策略**

- **内容容器约束**：桌面端不应让内容满屏铺开，设置最大内容宽度（如 1200px 或 1440px），超出部分作为双侧大留白。
- **纵向韵律**：宽屏模式下适当增加模块间距（增至 48-64px），用"横向空间换纵向呼吸感"。
- **多列布局的隐形分割**：列与列之间不使用竖线分割，仅依靠卡片自身的间距和悬浮阴影界定边界。

**6. 交互状态的隐形表达**

- **锚点导航**：导航选中态尽量避免使用下划线或包裹框，通过改变文字颜色（品牌色）、增加字重或在背景放置极微弱的方形色块来暗示。
- **按压反馈**：按钮按下时整体透明度变化（至 80%）或轻微缩放（Scale 0.97），利用动态视觉双重确认。

**7. 动效作为隐形的叙事线**

动效是弥补空间关系断裂的关键手段：

| 动效类型 | 时长 | 缓动曲线 | 说明 |
|:---|:---|:---|:---|
| 微交互（点击反馈） | 100ms – 150ms | `ease-out` / `cubic-bezier(0,0,0.2,1)` | 快速响应，即时确认 |
| 页面转场 | 250ms – 350ms | `cubic-bezier(0.4,0,0.2,1)` | 自然流畅，不拖沓 |
| 列表项出现 | 50ms – 100ms（错开） | `ease-out` | 轻盈入场，避免信息轰炸 |
| 弹层 / 浮层 | 200ms – 250ms | `cubic-bezier(0.4,0,0.2,1)` | 明确的空间层级建立 |

#### 三、适用边界与反模式

"去线留白"并非万能法则，以下场景中过度去线化会损害可用性：

- **高密度数据界面（B 端工具/金融）**：用户需要快速扫描比对数据，完全去线会导致视线在行间"漂移"。应审慎保留分割线，或采用**斑马纹**（交替极浅底色）辅助引导。
- **强区分与警示界面**：安全警告区、删除确认区、错误提示必须使用高饱和度的色彩背景或明显的边框，利用强对比打破常规的"宁静"，唤起用户警觉。
- **极简表单的长列表**：若完全去线，仅靠间距可能导致用户迷失当前位置，建议保留极细的引导线或输入字段左侧的极细轴线串联逻辑。

#### 四、可访问性（A11y）底线

- **对比度标准**：所有信息层级的前后景对比度不得低于 **WCAG AA**（正文文本至少 4.5:1，大文本至少 3:1）。
- **多维编码**：不依赖颜色或边框作为唯一的区分手段，必须结合**字重差异**、**空间位置**、**图标辅助**来传达信息，确保色盲用户或低视力用户也能准确获取内容。

### 2.3 设计令牌

- 令牌定义在 `<style>` 内的 `:root`（浅色）与 `[data-theme="dark"]`（深色）代码块中，**全部取值见「三、起步骨架」，不要另写一套**。
- **深色模式机制**：`<html>` 上的 `data-theme="dark"` 属性触发令牌整体反转，切换逻辑见骨架脚本。
- **自定义色彩**：需要新颜色时，先定义成新令牌再使用，禁止在样式中直接写色值；调整品牌色时只改令牌值，不改样式。

令牌清单（名称与用途）：

| 类别 | 令牌 | 用途 |
|------|------|------|
| 画布/表面 | `--bg` / `--sidebar` / `--card` / `--titlebar` | 页面底、侧栏、卡片、标题栏底色 |
| 交互底色 | `--hover` / `--divider` / `--input-bg` / `--input-bg-focus` | 悬停、按压、填充式输入框 |
| 强调色 | `--accent` / `--accent-bg` / `--accent-hover` / `--on-accent` | 主按钮、选中态、链接 |
| 语义色 | `--danger(-bg)` / `--warning(-bg)` / `--copy-success` | 错误、警告、成功 |
| 文字阶梯 | `--text-primary` / `--text-body` / `--text-auxiliary` / `--text-placeholder` | 用不透明度而非灰阶表达层级 |
| 动效 | `--ease` / `--duration-fast` / `--duration-normal` / `--duration-slow` | 统一缓动与时长 |
| 层级阴影 | `--shadow-card` / `--shadow-flyout` | 弥散阴影代替边框线表达 elevation |
| 几何/字体 | `--radius` / `--radius-sm` / `--font` / `--titlebar-h` | 圆角、字体栈、标题栏高度 |

---

## 三、起步骨架

> 以下是合规骨架（方式 A，自绘标题栏）：**在此填充你的内容，不要修改骨架的标题栏、令牌样式与脚本**。
> 填充位置：`<main class="content">` 内的 `<!-- 页面内容写在这里 -->` 处；需要新组件样式时在 `</style>` 前追加，新逻辑脚本在 `<script>` 内追加。
> 用户要求原生标题栏（方式 B）时，按 1.2 方式 B 的步骤删除标题栏相关部分。

```html
<!DOCTYPE html>
<html lang="zh-CN">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>（应用名称，与功能匹配）</title>
    <style>
        /* ═══════════ 设计令牌（浅色） ═══════════ */
        :root {
            --bg: #fcfcfc;
            --sidebar: #feffff;
            --card: #feffff;
            --titlebar: #feffff;
            --divider: #d9d9d9;
            --border-soft: color-mix(in srgb, #d9d9d9 40%, transparent);
            --hover: #e0e0e0;
            --code-bg: #e8e8e8;
            --code-text: #2b2b2b;
            --danger: #c42b1c;
            --danger-bg: rgba(196, 43, 28, 0.08);
            --warning: #b26b00;
            --warning-bg: rgba(255, 179, 0, 0.1);
            --copy-success: #107c10;

            --accent: #005fb8;
            --accent-bg: #e8f4fd;
            --accent-hover: #004a94;
            --on-accent: #ffffff;
            --static-white: #ffffff;

            --text-primary: rgba(0, 0, 0, 0.87);
            --text-body: rgba(0, 0, 0, 0.78);
            --text-auxiliary: rgba(0, 0, 0, 0.52);
            --text-placeholder: rgba(0, 0, 0, 0.3);

            --input-bg: rgba(0, 0, 0, 0.02);
            --input-bg-focus: rgba(0, 0, 0, 0.04);

            --ease: cubic-bezier(0.4, 0, 0.2, 1);
            --duration-fast: 100ms;
            --duration-normal: 200ms;
            --duration-slow: 300ms;

            --shadow-card: 0 2px 8px rgba(0, 0, 0, 0.06);
            --shadow-flyout: 0 2px 8px rgba(0, 0, 0, 0.14);

            --close-hover: #c42b1c;
            --close-active: #b3271a;

            --radius: 10px;
            --radius-sm: 6px;
            --font: "Noto Serif SC", "edit Han Serif SC", "SimSun", "宋体", serif;
            --titlebar-h: 40px;
        }

        /* ═══════════ 设计令牌（深色） ═══════════
           默认不启用：页面固定浅色主题；用户明确要求深色模式时启用本块（见规范 1.4） */
        [data-theme="dark"] {
            --bg: #202020;
            --sidebar: #232323;
            --card: #232323;
            --titlebar: #232323;
            --divider: #454545;
            --border-soft: color-mix(in srgb, #454545 40%, transparent);
            --hover: #3d3d3d;
            --code-bg: #2d2d2d;
            --code-text: #e8e8e8;
            --danger: #ff6b6b;
            --danger-bg: rgba(255, 107, 107, 0.1);
            --warning: #ffb84d;
            --warning-bg: rgba(255, 184, 77, 0.1);
            --copy-success: #4ade80;

            --accent: #60cdff;
            --accent-bg: #2a3f5c;
            --accent-hover: #8ad6ff;
            --on-accent: rgba(0, 0, 0, 0.87);

            --text-primary: rgba(255, 255, 255, 0.87);
            --text-body: rgba(255, 255, 255, 0.6);
            --text-auxiliary: rgba(255, 255, 255, 0.4);
            --text-placeholder: rgba(255, 255, 255, 0.25);

            --input-bg: rgba(255, 255, 255, 0.04);
            --input-bg-focus: rgba(255, 255, 255, 0.07);

            --shadow-card: 0 2px 8px rgba(0, 0, 0, 0.3);
            --shadow-flyout: 0 2px 8px rgba(0, 0, 0, 0.4);
        }

        /* ═══════════ 基础重置（骨架，勿改） ═══════════ */
        * { margin: 0; padding: 0; box-sizing: border-box; }
        html {
            -webkit-font-smoothing: antialiased;
            -moz-osx-font-smoothing: grayscale;
            background: var(--bg);
            overscroll-behavior: none;
        }
        body {
            font-family: var(--font);
            background: var(--bg);
            color: var(--text-primary);
            min-height: 100vh;
            overflow: hidden;            /* 必须：页面自身不滚动，滚动交给 .content */
            -webkit-user-select: none;
            user-select: none;
        }

        /* ═══════════ 标题栏（骨架，勿改；方式 B 时整段删除） ═══════════ */
        .titlebar {
            position: fixed;
            top: 0; left: 0; right: 0;
            height: var(--titlebar-h);
            display: flex;
            align-items: center;
            padding-left: 16px;
            background: var(--titlebar);
            -webkit-user-select: none;
            user-select: none;
            z-index: 1000;
        }
        .titlebar-title {
            font-size: 0.78rem;
            font-weight: 600;
            color: var(--text-body);
            letter-spacing: 0.01em;
            pointer-events: none;        /* 点击穿透，标题文字上也能拖拽 */
        }
        .window-controls {
            position: absolute;
            right: 0; top: 0; bottom: 0;
            display: flex;
            align-items: stretch;
        }
        .win-btn {
            width: 46px;
            border: none;
            cursor: pointer;
            padding: 0;
            background: transparent;
            transition: background var(--duration-fast) var(--ease);
            display: flex;
            align-items: center;
            justify-content: center;
            color: var(--text-body);
        }
        .win-btn:hover { background: var(--hover); }
        .win-btn:active { background: var(--divider); }
        .win-btn-close:hover { background: var(--close-hover); color: var(--static-white); }
        .win-btn-close:active { background: var(--close-active); color: var(--static-white); }
        .win-btn::after { content: ''; display: block; pointer-events: none; }
        .win-btn-close::after { content: '\2715'; font-size: 10px; font-family: 'Segoe UI', system-ui, sans-serif; }
        .win-btn-min::after {
            content: '';
            width: 10px;
            height: 1.5px;
            background: currentColor;
            border-radius: 1px;
        }
        .win-btn-max::after {
            content: '';
            width: 10px;
            height: 10px;
            border: 1.5px solid currentColor;
            border-radius: 1px;
            box-sizing: border-box;
        }
        /* 最大化后的还原图标：Windows 经典双层方块 */
        .win-btn.maximized { position: relative; }
        .win-btn.maximized::after {
            width: 8px; height: 8px;
            border: 1.5px solid currentColor;
            border-radius: 1px;
            box-sizing: border-box;
            transform: translate(-1px, 1px);
        }
        .win-btn.maximized::before {
            content: '';
            position: absolute;
            left: 50%; top: 50%;
            width: 8px; height: 8px;
            border-top: 1.5px solid currentColor;
            border-right: 1.5px solid currentColor;
            border-top-right-radius: 2px;
            box-sizing: border-box;
            transform: translate(calc(-50% + 2px), calc(-50% - 2px));
            pointer-events: none;
        }

        /* ═══════════ 内容区布局（骨架，勿改；方式 B 时改 height: 100vh / margin-top: 0） ═══════════ */
        .content {
            height: calc(100vh - var(--titlebar-h));
            margin-top: var(--titlebar-h);
            overflow-y: auto;
            overscroll-behavior: none;
            padding: 24px;
        }
        /* 宽屏内容宽度约束（设计语言：内容容器不铺满） */
        .content-inner { max-width: 1200px; margin: 0 auto; }

        /* 滚动条美化（细窄圆角） */
        .content::-webkit-scrollbar { width: 6px; }
        .content::-webkit-scrollbar-track { background: transparent; }
        .content::-webkit-scrollbar-thumb { background: var(--divider); border-radius: 3px; }

        /* ═══════════ 基础组件（可扩展） ═══════════ */
        .page-title { font-size: 1.15rem; font-weight: 700; margin-bottom: 4px; letter-spacing: -0.01em; }
        .page-desc { font-size: 0.78rem; color: var(--text-auxiliary); margin-bottom: 16px; }
        .card {
            background: var(--card);
            border-radius: var(--radius);
            padding: 16px;
            box-shadow: var(--shadow-card);
        }
        .card-spaced { margin-bottom: 16px; }
        .card-title-xs { font-size: 0.88rem; font-weight: 600; margin-bottom: 2px; }
        .card-subtitle { font-size: 0.7rem; color: var(--text-auxiliary); margin-bottom: 8px; }
        .btn {
            display: inline-flex;
            align-items: center;
            gap: 6px;
            padding: 8px 16px;
            border-radius: var(--radius-sm);
            font-size: 0.8rem;
            font-weight: 520;
            font-family: var(--font);
            cursor: pointer;
            border: none;
            transition: background var(--duration-fast) var(--ease), opacity var(--duration-fast) var(--ease), transform var(--duration-fast) var(--ease);
        }
        .btn:active { transform: scale(0.97); opacity: 0.8; }
        .btn-primary { background: var(--accent); color: var(--on-accent); }
        .btn-primary:hover { background: var(--accent-hover); }
        .btn-secondary { background: var(--hover); color: var(--text-primary); }
        .btn-secondary:hover { opacity: 0.8; }
        .form-group { margin-bottom: 16px; }
        .form-label { display: block; font-size: 0.78rem; color: var(--text-auxiliary); margin-bottom: 6px; }
        .form-input {
            width: 100%;
            padding: 8px 12px;
            border: none;
            border-radius: var(--radius-sm);
            background: var(--input-bg);
            color: var(--text-primary);
            font-family: var(--font);
            font-size: 0.85rem;
            outline: none;
            transition: background var(--duration-fast) var(--ease);
        }
        .form-input:focus { background: var(--input-bg-focus); }

        /* 动效降级 */
        @media (prefers-reduced-motion: reduce) {
            *, *::before, *::after {
                animation-duration: 0.01ms !important;
                animation-iteration-count: 1 !important;
                transition-duration: 0.01ms !important;
            }
        }
    </style>
</head>
<body>

    <!-- ═══════════ 标题栏（骨架，勿改；方式 B 时整段删除） ═══════════ -->
    <div class="titlebar" data-tauri-drag-region>
        <span class="titlebar-title">（应用名称，与 <title> 保持一致）</span>
        <div class="window-controls">
            <button class="win-btn win-btn-min" id="winMin" title="最小化" aria-label="最小化"></button>
            <button class="win-btn win-btn-max" id="winMax" title="最大化" aria-label="最大化"></button>
            <button class="win-btn win-btn-close" id="winClose" title="关闭" aria-label="关闭"></button>
        </div>
    </div>

    <!-- ═══════════ 内容区：在这里填充用户需求的功能 ═══════════ -->
    <main class="content">
        <div class="content-inner">
            <!-- 页面内容写在这里 -->
        </div>
    </main>

    <script>
    {
        /* ═══════════ 窗口控制（骨架，勿改；方式 B 时整段删除） ═══════════ */
        const titlebar = document.querySelector('.titlebar');
        const internals = window.__TAURI_INTERNALS__;

        if (!internals) {
            if (titlebar) titlebar.style.display = 'none';
            const content = document.querySelector('.content');
            if (content) { content.style.marginTop = '0'; content.style.height = '100vh'; }
        } else {
            const invoke = cmd => internals.invoke(cmd);

            document.getElementById('winMin').addEventListener('click', () => invoke('plugin:window|minimize'));
            document.getElementById('winMax').addEventListener('click', () => invoke('plugin:window|toggle_maximize'));
            // 关闭：默认直接关闭；用户明确要求"最小化到托盘"时：
            // <html> 加 data-tauri-tray 属性，并改为 invoke('plugin:window|hide')
            document.getElementById('winClose').addEventListener('click', () => invoke('plugin:window|close'));

            const maxBtn = document.getElementById('winMax');
            const updateMaxBtn = async () => {
                try {
                    const maximized = await invoke('plugin:window|is_maximized');
                    maxBtn.classList.toggle('maximized', maximized);
                    maxBtn.title = maximized ? '还原' : '最大化';
                } catch { /* 忽略 */ }
            };
            window.addEventListener('resize', updateMaxBtn);
            updateMaxBtn();
        }

        /* ═══════════ 你的业务逻辑写在这里 ═══════════ */
    }
    </script>
</body>
</html>
```

---

## 四、交付前自查清单

输出前逐条检查，全部通过才可交付：

- □ 单文件，无任何外链资源（外链 CSS / CDN / 外部图片 / 网络字体）
- □ 窗口形态正确：方式 A（标题栏带 `data-tauri-drag-region`、三按钮、脚本齐全）或方式 B（无任何标题栏残留代码）
- □ 关闭按钮命令正确：默认 `plugin:window|close`；仅用户明确要求托盘时用 `plugin:window|hide` + `<html>` 加 `data-tauri-tray`
- □ `body` 不滚动（`overflow: hidden`），滚动在 `.content`
- □ 所有颜色来自令牌，无写死色值；间距为 8 的整数倍；无多余边框线
- □ 深色模式：默认固定浅色主题、无切换入口；仅用户明确要求时提供切换且工作正常
- □ 默认右键菜单已屏蔽（默认行为）
- □ 普通浏览器中直接打开文件也能正常显示（方式 A 标题栏自动隐藏）
- □ 内容完整、无占位符残留（lorem / xxx / TODO / 示例文案）
- □ `<title>` 与标题栏文字一致，名称与功能匹配
