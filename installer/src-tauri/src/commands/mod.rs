// Copyright (c) 2026 青山公仔
// 轻壳 (Tauri Shell) is licensed under Mulan PSL v2.
// You can use this software according to the terms and conditions of the Mulan PSL v2.
// You may obtain a copy of Mulan PSL v2 at:
//          http://license.coscl.org.cn/MulanPSL2
// THIS SOFTWARE IS PROVIDED ON AN "AS IS" BASIS, WITHOUT WARRANTIES OF ANY KIND,
// EITHER EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO NON-INFRINGEMENT,
// MERCHANTABILITY OR FIT FOR A PARTICULAR PURPOSE.
// See the Mulan PSL v2 for more details.

pub mod elevate;
pub mod extract;
pub mod install;
pub mod process;
pub mod progress;
pub mod registry;
pub mod shortcut;
pub mod uninstall;

/// UTF-16 空结尾编码（PCWSTR 要求调用期间缓冲区保持存活）
pub(crate) fn wide(s: &str) -> Vec<u16> {
    s.encode_utf16().chain(std::iter::once(0)).collect()
}
