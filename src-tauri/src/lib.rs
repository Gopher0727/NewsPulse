mod config;
mod system;

use system::notice::Notifier;
use tauri::{
    menu::{MenuBuilder, MenuItemBuilder},
    tray::TrayIconBuilder,
    Manager,
};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_notification::init())
        .plugin(
            tauri_plugin_log::Builder::default()
                .level(log::LevelFilter::Info)
                .build(),
        )
        .setup(|app| {
            // 检查通知权限
            use tauri_plugin_notification::{NotificationExt, PermissionState};

            let mut permission_granted = app
                .handle()
                .notification()
                .permission_state()
                .unwrap_or(PermissionState::Denied)
                == PermissionState::Granted;
            if !permission_granted {
                permission_granted = app
                    .handle()
                    .notification()
                    .request_permission()
                    .unwrap_or(PermissionState::Denied)
                    == PermissionState::Granted;
            }
            log::info!("[notice] permission granted: {}", permission_granted);

            // 初始化通知器并注册到状态管理
            if permission_granted {
                let notifier = Notifier::new(app.handle().clone());
                notifier.setup_hide_on_close();

                // 测试通知，验证后删除
                notifier.push("test", "NewsPulse", "通知功能正常工作！");

                app.manage(notifier);
            }

            // 系统托盘
            let show_item = MenuItemBuilder::with_id("show", "显示窗口").build(app)?;
            let quit_item = MenuItemBuilder::with_id("quit", "退出").build(app)?;
            let menu = MenuBuilder::new(app)
                .items(&[&show_item, &quit_item])
                .build()?;

            let _tray = TrayIconBuilder::new()
                .icon(app.default_window_icon().unwrap().clone())
                .menu(&menu)
                .tooltip("NewsPulse")
                .on_menu_event(|app, event| match event.id().as_ref() {
                    "show" => {
                        if let Some(w) = app.get_webview_window("main") {
                            let _ = w.show();
                            let _ = w.set_focus();
                        }
                    }
                    "quit" => app.exit(0),
                    _ => {}
                })
                .build(app)?;

            Ok(())
        })
        .build(tauri::generate_context!())
        .expect("error while building tauri application")
        .run(|app, event| {
            if let tauri::RunEvent::Reopen { .. } = event {
                if let Some(w) = app.get_webview_window("main") {
                    let _ = w.show();
                    let _ = w.set_focus();
                }
            }
        });
}
