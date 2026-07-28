// Copyright (c) 2026 青山公仔
// 轻壳 (Tauri Shell) is licensed under Mulan PSL v2.
// You can use this software according to the terms and conditions of the Mulan PSL v2.
// You may obtain a copy of Mulan PSL v2 at:
//          http://license.coscl.org.cn/MulanPSL2
// THIS SOFTWARE IS PROVIDED ON AN "AS IS" BASIS, WITHOUT WARRANTIES OF ANY KIND,
// EITHER EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO NON-INFRINGEMENT,
// MERCHANTABILITY OR FIT FOR A PARTICULAR PURPOSE.
// See the Mulan PSL v2 for more details.

import ReactDOM from "react-dom/client";
import App from "./App";
import "./index.css";

// 主题跟随系统：切换 documentElement 的 .dark 类（安装器无手动切换）
const media = window.matchMedia("(prefers-color-scheme: dark)");
const applyTheme = () => {
  document.documentElement.classList.toggle("dark", media.matches);
};
applyTheme();
media.addEventListener("change", applyTheme);

// 屏蔽浏览器加速键与右键菜单（仅生产构建，开发预览保留 F5/F12 便于调试）——
// 安装向导中刷新/缩放/历史导航等行为会暴露 Web 本质且可能中断安装流程
if (import.meta.env.PROD) {
  document.addEventListener("contextmenu", (e) => e.preventDefault());
  const BLOCKED_CTRL_KEYS = ["r", "p", "u", "f", "g", "j", "+", "-", "=", "0"];
  document.addEventListener(
    "keydown",
    (e) => {
      const key = e.key.toLowerCase();
      if (
        key === "f5" || key === "f3" || key === "f12" ||
        (e.ctrlKey && e.shiftKey && ["i", "j", "c"].includes(key)) ||
        (e.ctrlKey && !e.altKey && BLOCKED_CTRL_KEYS.includes(key)) ||
        (e.altKey && (key === "arrowleft" || key === "arrowright"))
      ) {
        e.preventDefault();
      }
    },
    { capture: true },
  );
  document.addEventListener(
    "wheel",
    (e) => {
      if (e.ctrlKey) e.preventDefault();
    },
    { passive: false, capture: true },
  );
  document.addEventListener("mouseup", (e) => {
    if (e.button === 3 || e.button === 4) e.preventDefault();
  });
}

// 故意不用 React.StrictMode：安装是一次性强副作用（写注册表/复制文件/可能触发 UAC），
// StrictMode 的 effect 双调用会与“安装仅执行一次”的需求相冲突
ReactDOM.createRoot(document.getElementById("root")!).render(<App />);
