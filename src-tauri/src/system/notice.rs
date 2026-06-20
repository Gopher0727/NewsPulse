use crate::config::AppConfig;

use std::{collections::HashMap, sync::Mutex, time::Instant};

use tauri::{AppHandle, Manager};

pub struct Notifier {
    app: AppHandle,
    // 记录每条 Feed 上次推送时间
    last_push: Mutex<HashMap<String, Instant>>,
}

impl Notifier {
    pub fn new(app: AppHandle) -> Self {
        Self {
            app,
            last_push: Mutex::new(HashMap::new()),
        }
    }

    // 推送通知，带冷却期控制
    pub fn push(&self, feed_id: &str, title: &str, body: &str) {
        let mut map = self.last_push.lock().unwrap();

        if let Some(t) = map.get(feed_id) {
            if t.elapsed() < AppConfig::global().cooldown() {
                return;
            }
        }

        // macOS 用 osascript（不需要签名，dev/release 都能弹通知）
        #[cfg(target_os = "macos")]
        super::notice_macos::send_notification(title, body);

        // 其他平台用 tauri-plugin-notification
        #[cfg(not(target_os = "macos"))]
        {
            use tauri_plugin_notification::NotificationExt;
            self.app
                .notification()
                .builder()
                .title(title)
                .body(body)
                .show()
                .ok();
        }

        map.insert(feed_id.to_string(), Instant::now());
    }

    // 拦截关闭事件：隐藏窗口而非销毁，保持后台运行
    pub fn setup_hide_on_close(&self) {
        if let Some(w) = self.app.get_webview_window("main") {
            let app = self.app.clone();
            w.on_window_event(move |event| {
                if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                    api.prevent_close();
                    if let Some(w) = app.get_webview_window("main") {
                        let _ = w.hide();
                    }
                }
            });
        }
    }
}
