// Copyright (c) 2026 青山公仔
// 轻壳 (Tauri Shell) is licensed under Mulan PSL v2.
// You can use this software according to the terms and conditions of the Mulan PSL v2.
// You may obtain a copy of Mulan PSL v2 at:
//          http://license.coscl.org.cn/MulanPSL2
// THIS SOFTWARE IS PROVIDED ON AN "AS IS" BASIS, WITHOUT WARRANTIES OF ANY KIND,
// EITHER EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO NON-INFRINGEMENT,
// MERCHANTABILITY OR FIT FOR A PARTICULAR PURPOSE.
// See the Mulan PSL v2 for more details.

const { minify } = require("html-minifier-terser");
const fs = require("fs");

const OPTIONS = {
  collapseWhitespace: true,
  removeComments: true,
  minifyCSS: true,
  minifyJS: true,
};

// 主页面 + 托盘菜单页（白名单式拷贝，dist 内其余文件不进打包产物）
const PAGES = ["index.html", "tray-menu.html"];

fs.mkdirSync("dist-min", { recursive: true });
Promise.all(
  PAGES.map(async (name) => {
    const html = fs.readFileSync(`dist/${name}`, "utf8");
    const r = await minify(html, OPTIONS);
    fs.writeFileSync(`dist-min/${name}`, r);
    console.log(`Minified ${name}:`, html.length, "->", r.length);
  }),
).then(() => {
  // 非 HTML 静态资源直接拷贝（关于页 logo 等）
  fs.copyFileSync("dist/logo.svg", "dist-min/logo.svg");
});
