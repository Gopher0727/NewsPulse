use std::process::Command;

/// 通过 osascript 发送通知（macOS 专用）
/// 比 UNUserNotificationCenter 简单，不需要签名
pub fn send_notification(title: &str, body: &str) {
    let script = format!(
        r#"display notification "{}" with title "{}""#,
        body.replace('\\', "\\\\").replace('"', "\\\""),
        title.replace('\\', "\\\\").replace('"', "\\\""),
    );

    match Command::new("osascript").arg("-e").arg(&script).output() {
        Ok(output) => {
            if !output.status.success() {
                log::error!(
                    "[macos_notify] osascript failed: {}",
                    String::from_utf8_lossy(&output.stderr)
                );
            }
        }
        Err(e) => {
            log::error!("[macos_notify] failed to run osascript: {e}");
        }
    }
}
