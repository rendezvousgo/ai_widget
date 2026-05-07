mod claude_usage;
mod oauth_usage;
mod api_keys;
mod provider_probes;
mod login_status;
mod paths;

use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use tauri::{Manager, PhysicalSize, WindowEvent};

const ASPECT_W: f64 = 360.0;
const ASPECT_H: f64 = 220.0;

static SKIP_RESIZE: AtomicBool = AtomicBool::new(false);
static LAST_W: AtomicU32 = AtomicU32::new(360);
static LAST_H: AtomicU32 = AtomicU32::new(220);

#[cfg(target_os = "linux")]
fn bypass_kwin_compositor_for_window() {
    // After app starts, find any unnamed visible windows that look like the widget
    // and set _NET_WM_BYPASS_COMPOSITOR=1 via xprop. Cheap and avoids extra deps.
    std::thread::spawn(|| {
        std::thread::sleep(std::time::Duration::from_millis(800));
        let _ = std::process::Command::new("sh")
            .arg("-c")
            .arg(r#"
                for id in $(xwininfo -root -tree 2>/dev/null | awk '/^     0x/ {print $1}'); do
                    info=$(xwininfo -id "$id" 2>/dev/null);
                    ms=$(echo "$info" | awk '/Map State/ {print $3}');
                    w=$(echo "$info" | awk '/Width:/ {print $2}');
                    h=$(echo "$info" | awk '/Height:/ {print $2}');
                    if [ "$ms" = "IsViewable" ] && [ "$w" -ge 200 ] && [ "$w" -le 2000 ] \
                       && [ "$h" -ge 100 ] && [ "$h" -le 1500 ]; then
                        nm=$(xprop -id "$id" WM_CLASS 2>/dev/null);
                        if echo "$nm" | grep -q "not found"; then
                            xprop -id "$id" -f _NET_WM_BYPASS_COMPOSITOR 32c \
                                -set _NET_WM_BYPASS_COMPOSITOR 1 2>/dev/null || true;
                            xprop -id "$id" -f _KDE_NET_WM_SHADOW 32c \
                                -remove _KDE_NET_WM_SHADOW 2>/dev/null || true;
                        fi;
                    fi;
                done
            "#)
            .output();
    });
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_autostart::init(
            tauri_plugin_autostart::MacosLauncher::LaunchAgent,
            None,
        ))
        .setup(|app| {
            if cfg!(debug_assertions) {
                app.handle().plugin(
                    tauri_plugin_log::Builder::default()
                        .level(log::LevelFilter::Info)
                        .build(),
                )?;
            }
            #[cfg(target_os = "linux")]
            bypass_kwin_compositor_for_window();
            Ok(())
        })
        .on_window_event(|window, event| {
            if let WindowEvent::Resized(size) = event {
                // aspect-ratio lock only for the widget window
                if window.label() != "main" {
                    return;
                }
                if SKIP_RESIZE.swap(false, Ordering::SeqCst) {
                    LAST_W.store(size.width, Ordering::SeqCst);
                    LAST_H.store(size.height, Ordering::SeqCst);
                    return;
                }
                if size.width == 0 || size.height == 0 {
                    return;
                }
                let prev_w = LAST_W.load(Ordering::SeqCst);
                let prev_h = LAST_H.load(Ordering::SeqCst);
                let dw = (size.width as i64 - prev_w as i64).abs();
                let dh = (size.height as i64 - prev_h as i64).abs();

                let (target_w, target_h) = if dw >= dh {
                    let h = (size.width as f64 * ASPECT_H / ASPECT_W).round() as u32;
                    (size.width, h)
                } else {
                    let w = (size.height as f64 * ASPECT_W / ASPECT_H).round() as u32;
                    (w, size.height)
                };

                if target_w != size.width || target_h != size.height {
                    SKIP_RESIZE.store(true, Ordering::SeqCst);
                    let _ = window.set_size(PhysicalSize::new(target_w, target_h));
                }
                LAST_W.store(target_w, Ordering::SeqCst);
                LAST_H.store(target_h, Ordering::SeqCst);
            }
        })
        .invoke_handler(tauri::generate_handler![
            claude_usage::get_claude_usage,
            claude_usage::get_claude_daily_tokens,
            claude_usage::claude_trend_diag,
            oauth_usage::get_oauth_usage,
            oauth_usage::get_usage_history,
            api_keys::list_api_keys,
            api_keys::set_api_key,
            provider_probes::probe_provider,
            login_status::claude_login_status,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
