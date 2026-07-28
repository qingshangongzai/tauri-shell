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
const html = fs.readFileSync("dist/index.html", "utf8");
minify(html, {
  collapseWhitespace: true,
  removeComments: true,
  minifyCSS: true,
  minifyJS: true,
}).then((r) => {
  fs.mkdirSync("dist-min", { recursive: true });
  fs.writeFileSync("dist-min/index.html", r);
  // 非 HTML 静态资源直接拷贝（关于页 logo 等）
  fs.copyFileSync("dist/logo.svg", "dist-min/logo.svg");
  console.log("Minified:", html.length, "->", r.length);
});
