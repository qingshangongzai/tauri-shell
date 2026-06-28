const fs = require("fs");
const path = require("path");

// 读取 tauri.conf.json 中的 version
const tauriConf = JSON.parse(fs.readFileSync("src-tauri/tauri.conf.json", "utf8"));
const version = tauriConf.version;

// 同步到 package.json
const pkgPath = "package.json";
const pkg = JSON.parse(fs.readFileSync(pkgPath, "utf8"));
pkg.version = version;
fs.writeFileSync(pkgPath, JSON.stringify(pkg, null, 2) + "\n");

// 同步到 Cargo.toml（使用正则替换）
const cargoPath = path.join("src-tauri", "Cargo.toml");
let cargo = fs.readFileSync(cargoPath, "utf8");
cargo = cargo.replace(
  /^version\s*=\s*".*"/m,
  `version = "${version}"`
);
fs.writeFileSync(cargoPath, cargo);

console.log(`Version synced: ${version}`);
