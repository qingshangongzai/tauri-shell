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

// 故意不用 React.StrictMode：安装是一次性强副作用（写注册表/复制文件/可能触发 UAC），
// StrictMode 的 effect 双调用会与“安装仅执行一次”的需求相冲突
ReactDOM.createRoot(document.getElementById("root")!).render(<App />);
