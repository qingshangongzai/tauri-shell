// Copyright (c) 2026 青山公仔
// 轻壳 (Tauri Shell) is licensed under Mulan PSL v2.
// You can use this software according to the terms and conditions of the Mulan PSL v2.
// You may obtain a copy of Mulan PSL v2 at:
//          http://license.coscl.org.cn/MulanPSL2
// THIS SOFTWARE IS PROVIDED ON AN "AS IS" BASIS, WITHOUT WARRANTIES OF ANY KIND,
// EITHER EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO NON-INFRINGEMENT,
// MERCHANTABILITY OR FIT FOR A PARTICULAR PURPOSE.
// See the Mulan PSL v2 for more details.

#[tauri::command]
fn get_file_size(path: String) -> u64 {
    std::fs::metadata(&path).map(|m| m.len()).unwrap_or(0)
}

// ═══════════ 托盘（不需要托盘时按 README「按需移除托盘」小节整块删除） ═══════════

/// 挂托盘图标后销毁窗口不会退出进程，须显式退出（托盘菜单「退出」调用）
#[tauri::command]
fn exit_app(app: tauri::AppHandle) {
    app.exit(0);
}

/// 还原并前置主窗口（托盘左键 / 托盘菜单「打开主界面」调用）
#[tauri::command]
fn show_main_window(app: tauri::AppHandle) {
    use tauri::Manager;
    if let Some(w) = app.get_webview_window("main") {
        let _ = w.unminimize();
        let _ = w.show();
        let _ = w.set_focus();
    }
}

/// 托盘右键菜单定位：可见卡片左下角贴光标（向右上展开），
/// 靠近屏幕边缘时夹取推回，保证卡片完整可见。
/// DPI：定位用物理像素，尺寸由前端 LogicalSize 设置。
#[cfg(desktop)]
fn position_menu_window(
    app: &tauri::AppHandle,
    win: &tauri::WebviewWindow,
    cursor: tauri::PhysicalPosition<f64>,
) {
    /// 与前端 tray-menu.html 的透明投影边距一致（逻辑像素）
    const SHADOW_MARGIN: f64 = 16.0;
    let Ok(size) = win.outer_size() else { return };
    let shadow = (SHADOW_MARGIN * win.scale_factor().unwrap_or(1.0)).round() as i32;
    // 窗口尺寸含透明边距，需补偿才能让可见卡片边缘贴光标
    let mut x = cursor.x as i32 - shadow;
    let mut y = cursor.y as i32 - size.height as i32 + shadow;
    if let Ok(Some(m)) = app.monitor_from_point(cursor.x, cursor.y) {
        let (mp, ms) = (m.position(), m.size());
        // 夹取时同样排除透明边距，保证卡片完整可见
        x = x.clamp(mp.x - shadow, mp.x + ms.width as i32 - size.width as i32 + shadow);
        y = y.clamp(mp.y - shadow, mp.y + ms.height as i32 - size.height as i32 + shadow);
    }
    let _ = win.set_position(tauri::PhysicalPosition::new(x, y));
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            // ── 托盘图标注册（不需要托盘时整块删除） ──
            #[cfg(desktop)]
            {
                use tauri::{
                    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
                    Manager, WindowEvent,
                };

                // 菜单窗口失焦即隐藏（预建复用，勿销毁）
                if let Some(menu_win) = app.get_webview_window("tray-menu") {
                    let w = menu_win.clone();
                    menu_win.on_window_event(move |e| {
                        if let WindowEvent::Focused(false) = e {
                            let _ = w.hide();
                        }
                    });
                }

                // 不挂原生 .menu()，右键弹出自绘的 tray-menu Webview 窗口
                let tooltip = app.config().product_name.clone().unwrap_or_default();
                TrayIconBuilder::new()
                    .icon(app.default_window_icon().expect("图标缺失").clone())
                    .tooltip(tooltip)
                    .on_tray_icon_event(|tray, event| {
                        let app = tray.app_handle();
                        if let TrayIconEvent::Click {
                            button,
                            button_state: MouseButtonState::Up,
                            position,
                            ..
                        } = event
                        {
                            match button {
                                MouseButton::Left => show_main_window(app.clone()),
                                MouseButton::Right => {
                                    if let Some(m) = app.get_webview_window("tray-menu") {
                                        position_menu_window(app, &m, position);
                                        let _ = m.show();
                                        let _ = m.set_focus();
                                    }
                                }
                                _ => {}
                            }
                        }
                    })
                    .build(app)?;
            }
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            get_file_size,
            exit_app,
            show_main_window
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
