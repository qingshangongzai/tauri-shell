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
  console.log("Minified:", html.length, "->", r.length);
});
