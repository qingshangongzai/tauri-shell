---
name: ui-animation
description: "设计、实现、审查 UI 动效：CSS transition、keyframes、spring、手势、拖拽、缓动。当用户说'加动画'、'让这更流畅'、'审查动画'、'添加滑动手势'时调用。"
---

# UI Animation

> 编码规则（令牌、间距标尺、动效令牌要求等）以根目录《开发规范.md》为唯一真源；本 skill 只负责动效实现方法与审查流程，冲突时以它为准。

**属于：** 设计、实现、审查、调试 UI 动效（spring、手势、拖拽、缓动、CSS transition、keyframes）
**不属于：** 选择整体视觉方向/调色板/排版，审查整页 UI 质量

## 核心规则

- 动画目的：反馈、方向感、连续性或刻意的愉悦。如果只为"好看"且用户频繁看到，不要加
- **绝不**为键盘触发的操作做动画（快捷键、方向键导航、Tab/焦点切换），重复频繁会让操作感觉变慢
- 优先 CSS transition 处理可中断 UI：keyframes 中断后从头开始，transition 会重定向。keyframes 仅用于预定序列
- 实现优先级：CSS transition > WAAPI > CSS keyframes > JS（`requestAnimationFrame`）。负载下 CSS 保持流畅，JS 会掉帧
- 非对称时间：偶发交互可稍慢进入、快速退出。高频短暂 UI（hover 高亮、popover、面板切换）反转：即时进入（0ms），短暂淡出（100-150ms）
- 使用 `@starting-style` 处理 DOM 插入；不支持处回退到 `data-mounted` 属性
- 小幅 `filter: blur(2px)` 可隐藏内容切换间的粗糙交叉淡入

## 动效设计原则

- **连续性优于瞬移。** 两状态共有的元素原地过渡；从元素所在位置展开，而非淡入新实例。绝不复制持久元素或在共享组件的视图间硬切
- **方向性动效匹配位置。** Tab 和轮播过渡沿空间布局方向（左到右前进，右到左后退）
- **从触发器展开。** 覆盖层、托盘、面板从打开它们的元素向外展开；通用屏幕中心入场破坏空间定向
- **成对状态一起动画。** 打开有动画，关闭也要有。hover 有动效，focus 和 pressed 状态也要等效反馈
- **愉悦感与频率成反比。** 越少见的交互越多个性；高频操作必须不可见
- **动效增强感知速度。** 平滑过渡比硬切换感觉更快，即使实际加载时间相同

## 动画哪些属性

- 位移：仅 `transform` 和 `opacity`，它们跳过布局和绘制
- 状态反馈：`color`、`background-color`、`opacity` 可接受
- **绝不**动画布局属性（`width`、`height`、`top`、`left`），每帧触发布局重算
- **绝不**使用 `transition: all`，明确列出属性
- 避免核心交互使用 `filter` 动画；无法避免时保持 blur ≤ 20px
- SVG：在 `<g>` 包裹上设置 `transform-box: fill-box; transform-origin: center`
- `transform: scale()` 也会缩放子元素，这是按下反馈的特性，但如果内部元素需保持固定尺寸要考虑到
- 主题切换时禁用过渡：`[data-theme="dark"]` 切换期间给根元素临时挂类禁用 transition（如 `* { transition: none !important }`），切换完成即移除

## 缓动默认值

**本项目铁律**：动效时长与缓动必须使用项目令牌 `--duration-*` / `--ease`（`dist/index.html` 的 `:root`，见《开发规范.md》「前端规范」检查项 2），下表只用于**选择合适的量级与曲线**——选好后取最接近的项目令牌；令牌不够用时先补令牌（三处同步）再使用。

| 元素 | 时长 | 缓动 |
|------|------|------|
| 按钮按下反馈 | 100-160ms | `cubic-bezier(0.22, 1, 0.36, 1)` |
| 工具提示、小弹出框 | 125-200ms | `ease-out` 或入场曲线 |
| 下拉菜单、选择器 | 150-250ms | `cubic-bezier(0.22, 1, 0.36, 1)` |
| 模态框、抽屉 | 200-350ms | `cubic-bezier(0.22, 1, 0.36, 1)` |
| 屏幕移动/滑动 | 200-300ms | `cubic-bezier(0.25, 1, 0.5, 1)` |
| 页面过渡 | 250-400ms | 入场或移动曲线 |
| 简单 hover（颜色/透明度） | 200ms | `ease` |
| 插画/营销动效 | 最多 1000ms | Spring 或自定义 |

日常 UI 保持在 300ms 以内；时长随距离缩放（全屏滑动可超 300ms，6px 工具提示偏移保持 150ms 以内）。

**命名曲线：**
- **Enter：** `cubic-bezier(0.22, 1, 0.36, 1)` — 入场和基于 transform 的 hover
- **Move：** `cubic-bezier(0.25, 1, 0.5, 1)` — 滑动、抽屉、面板
- **Drawer (iOS 风格)：** `cubic-bezier(0.32, 0.72, 0, 1)`

避免 UI 使用 `ease-in`：开始慢，元素滞后于用户操作，感觉迟钝。优先使用 [easing.dev](https://easing.dev/) 的自定义曲线。

## 空间与序列

- 为 popover 设置 `transform-origin` 在触发点；模态框保持 `center`
- 对话框/菜单从约 `scale(0.85-0.9)` 开始。绝不 `scale(0)`：无中生有
- 逐个元素延迟 30-50ms 入场；总延迟控制在 300ms 以内。按视觉重要性变化时间，最重要元素先出场
- **成对元素规则：** 一起动画的元素必须共享缓动和时长

## 无障碍

- 每个动画须有 `prefers-reduced-motion: reduce` 路径：禁用 transform/keyframe 动效，保留即时状态变化或仅透明度淡入淡出
- 将 hover 动画限制在 `@media (hover: hover) and (pointer: fine)` 之后，否则触摸设备会在点击时重放 hover
- 直接操作期间，元素锁定在指针上，无缓动；释放后再加缓动

```css
/* 必须包含的 reduced motion 防护 */
@media (prefers-reduced-motion: reduce) {
  *, *::before, *::after {
    animation-duration: 0.01ms !important;
    animation-iteration-count: 1 !important;
    transition-duration: 0.01ms !important;
  }
}
```

> 上面的 `@media` 是**能力查询**（询问设备是否偏好减少动效），不属于《开发规范.md》禁止的**响应式断点**（`min-width`/`max-width`），可以使用。

## 性能

- 使用 `IntersectionObserver` 暂停屏幕外的循环动画
- `will-change` 仅在重度动效期间切换，仅用于 `transform`/`opacity`，用完移除
- 不要通过容器上的 CSS 变量驱动拖拽动画，直接在被移动元素上设置 `transform`
- 暂停离屏动画减少 GPU 消耗

## 反模式

- 无用户触发即挂载动画：意外动效让人迷失
- 拖拽边界硬停止：施加摩擦力/阻尼使移动减弱
- 混合使用 Motion `x`/`y` 与手写 `transform`：两者都写 `transform`，会互相覆盖
- 同时动画容器和延迟其子元素：每容器选一个入场方式
- 在频繁触发元素上使用 keyframes（toast、列表项）：中断后从头开始；用 CSS transition
- 首工具提示打开后，后续同组工具提示还有动画：应即时打开

## 工作流程

```
动画进度：
- [ ] 步骤 1：判断该交互是否应做动画
- [ ] 步骤 2：选择目的、缓动和时长（取项目 --duration-*/--ease 令牌）
- [ ] 步骤 3：选择实现方式
- [ ] 步骤 4：实现动画
- [ ] 步骤 5：验证时间、中断和设备行为
```

1. 回答四个问题：做动画？目的？缓动？速度？
2. 从缓动默认值表中选择时长量级，落到项目令牌
3. 选择实现：CSS transition > WAAPI > spring > keyframe > JS
4. 实现动画
5. 审查时严格验证

## 验证

提供每项检查的证据（DevTools 观察，非"看起来不错"）：

- Grep diff 查找布局属性 transition（`width`、`height`、`top`、`left`）和 `transition: all`
- 快速重复切换组件，确认 transition 重定向而非从头开始
- DevTools Animations 面板减速到 10% 检查时间和 `transform-origin` 问题
- 模拟 `prefers-reduced-motion: reduce`（DevTools Rendering 面板），确认每个动画都有 reduced 路径
- 确认 `will-change` 在动画周围切换，非永久设置
- 循环动画离屏时暂停
- 在真实设备上测试触摸交互
