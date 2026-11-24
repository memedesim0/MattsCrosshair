#![windows_subsystem = "windows"]

mod config;
mod draw;
mod image_loader;
mod monitor;
mod settings_gui;
mod tray;

use config::{Color, CrosshairConfig, CrosshairStyle};
use draw::render_crosshair;
use settings_gui::{notify_overlay_update, notify_settings_update, open_image_dialog, set_overlay_hwnd, show_settings_window, WM_APP_UPDATE};
use tray::*;
use windows::{
    core::*,
    Win32::Foundation::*,
    Win32::Graphics::Gdi::*,
    Win32::System::LibraryLoader::GetModuleHandleW,
    Win32::UI::Shell::ShellExecuteW,
    Win32::UI::WindowsAndMessaging::*,
};
use std::cell::RefCell;
use std::mem;

// Canvas size of 480x480 supports crosshairs and images up to 200px comfortably
const OVERLAY_CANVAS_SIZE: i32 = 480;

thread_local! {
    static CONFIG: RefCell<CrosshairConfig> = RefCell::new(CrosshairConfig::load());
    static APP_INSTANCE: RefCell<HINSTANCE> = RefCell::new(HINSTANCE(std::ptr::null_mut()));
}

pub fn get_config() -> CrosshairConfig {
    CONFIG.with(|c| c.borrow().clone())
}

pub fn set_config(cfg: CrosshairConfig) {
    let _ = cfg.save();
    CONFIG.with(|c| *c.borrow_mut() = cfg);
}

fn to_wide(s: &str) -> Vec<u16> {
    s.encode_utf16().chain(std::iter::once(0)).collect()
}

/// Creates a Windows HICON directly from the embedded thick red logo with white outline
pub unsafe fn create_logo_icon() -> Option<HICON> {
    let png_bytes = include_bytes!("../assets/logo.png");
    let img = image::load_from_memory(png_bytes).ok()?;
    let icon_size = 32u32;
    let resized = img.resize_exact(icon_size, icon_size, image::imageops::FilterType::Lanczos3).to_rgba8();

    let screen_dc = GetDC(None);
    let mut bmi: BITMAPINFO = mem::zeroed();
    bmi.bmiHeader.biSize = mem::size_of::<BITMAPINFOHEADER>() as u32;
    bmi.bmiHeader.biWidth = icon_size as i32;
    bmi.bmiHeader.biHeight = -(icon_size as i32); // Top-down
    bmi.bmiHeader.biPlanes = 1;
    bmi.bmiHeader.biBitCount = 32;
    bmi.bmiHeader.biCompression = BI_RGB.0;

    let mut bits: *mut std::ffi::c_void = std::ptr::null_mut();
    let hbm_color = CreateDIBSection(screen_dc, &bmi, DIB_RGB_COLORS, &mut bits, None, 0).ok()?;

    if !bits.is_null() {
        let mut bgra = Vec::with_capacity((icon_size * icon_size * 4) as usize);
        for p in resized.pixels() {
            let (r, g, b, a) = (p[0], p[1], p[2], p[3]);
            let a_f = a as f32 / 255.0;
            bgra.push((b as f32 * a_f).round() as u8);
            bgra.push((g as f32 * a_f).round() as u8);
            bgra.push((r as f32 * a_f).round() as u8);
            bgra.push(a);
        }
        std::ptr::copy_nonoverlapping(bgra.as_ptr(), bits as *mut u8, bgra.len());
    }

    let hbm_mask = CreateBitmap(icon_size as i32, icon_size as i32, 1, 1, None);

    let mut ii = ICONINFO {
        fIcon: true.into(),
        xHotspot: 0,
        yHotspot: 0,
        hbmMask: hbm_mask,
        hbmColor: hbm_color,
    };

    let hicon = CreateIconIndirect(&mut ii).ok();

    let _ = DeleteObject(hbm_mask);
    let _ = DeleteObject(hbm_color);
    let _ = ReleaseDC(None, screen_dc);

    hicon
}

unsafe fn update_overlay(hwnd: HWND) {
    let cfg = get_config();
    let (center_x, center_y) = monitor::get_monitor_center(cfg.monitor_index);
    let overlay_size = OVERLAY_CANVAS_SIZE;
    let win_x = center_x - overlay_size / 2;
    let win_y = center_y - overlay_size / 2;

    // Reposition the overlay to the center of the chosen monitor
    let _ = SetWindowPos(
        hwnd,
        HWND_TOPMOST,
        win_x,
        win_y,
        overlay_size,
        overlay_size,
        SWP_NOACTIVATE | SWP_SHOWWINDOW,
    );

    let mut buffer = vec![0u8; (overlay_size * overlay_size * 4) as usize];
    render_crosshair(&mut buffer, overlay_size as u32, overlay_size as u32, &cfg);

    let screen_dc = GetDC(None);
    let mem_dc = CreateCompatibleDC(screen_dc);

    let mut bmi: BITMAPINFO = mem::zeroed();
    bmi.bmiHeader.biSize = mem::size_of::<BITMAPINFOHEADER>() as u32;
    bmi.bmiHeader.biWidth = overlay_size;
    bmi.bmiHeader.biHeight = -overlay_size; // Top-down DIB
    bmi.bmiHeader.biPlanes = 1;
    bmi.bmiHeader.biBitCount = 32;
    bmi.bmiHeader.biCompression = BI_RGB.0;

    let mut bits: *mut std::ffi::c_void = std::ptr::null_mut();
    if let Ok(hbitmap) = CreateDIBSection(screen_dc, &bmi, DIB_RGB_COLORS, &mut bits, None, 0) {
        if !bits.is_null() {
            std::ptr::copy_nonoverlapping(buffer.as_ptr(), bits as *mut u8, buffer.len());
        }

        let old_bmp = SelectObject(mem_dc, hbitmap);

        let pt_dst = POINT { x: win_x, y: win_y };
        let size = SIZE {
            cx: overlay_size,
            cy: overlay_size,
        };
        let pt_src = POINT { x: 0, y: 0 };
        let blend = BLENDFUNCTION {
            BlendOp: AC_SRC_OVER as u8,
            BlendFlags: 0,
            SourceConstantAlpha: cfg.opacity,
            AlphaFormat: AC_SRC_ALPHA as u8,
        };

        let _ = UpdateLayeredWindow(
            hwnd,
            screen_dc,
            Some(&pt_dst),
            Some(&size),
            mem_dc,
            Some(&pt_src),
            COLORREF(0),
            Some(&blend),
            ULW_ALPHA,
        );

        SelectObject(mem_dc, old_bmp);
        let _ = DeleteObject(hbitmap);
    }

    let _ = DeleteDC(mem_dc);
    let _ = ReleaseDC(None, screen_dc);
}

unsafe extern "system" fn overlay_wnd_proc(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    match msg {
        WM_CREATE => {
            add_tray_icon(hwnd);
            set_overlay_hwnd(hwnd);
            update_overlay(hwnd);
            LRESULT(0)
        }

        WM_APP_UPDATE => {
            update_overlay(hwnd);
            LRESULT(0)
        }

        WM_DISPLAYCHANGE => {
            update_overlay(hwnd);
            notify_settings_update();
            LRESULT(0)
        }

        WM_TRAY => {
            let event = lparam.0 as u32;
            if event == WM_RBUTTONUP || event == WM_CONTEXTMENU {
                show_context_menu(hwnd);
            } else if event == WM_LBUTTONDBLCLK || event == WM_LBUTTONUP {
                APP_INSTANCE.with(|inst| {
                    show_settings_window(*inst.borrow());
                });
            }
            LRESULT(0)
        }

        WM_COMMAND => {
            let id = (wparam.0 & 0xFFFF) as u32;
            let mut cfg = get_config();
            let mut changed = true;

            if (ID_TRAY_MONITOR_BASE..ID_TRAY_MONITOR_BASE + 16).contains(&id) {
                let mon_idx = (id - ID_TRAY_MONITOR_BASE) as usize;
                cfg.monitor_index = mon_idx;
            } else {
                match id {
                    ID_TRAY_SETTINGS => {
                        APP_INSTANCE.with(|inst| {
                            show_settings_window(*inst.borrow());
                        });
                        changed = false;
                    }
                    ID_TRAY_OPEN_FOLDER => {
                        let folder = CrosshairConfig::custom_crosshairs_dir();
                        let folder_str = to_wide(&folder.to_string_lossy());
                        let op = to_wide("open");
                        let _ = ShellExecuteW(None, PCWSTR(op.as_ptr()), PCWSTR(folder_str.as_ptr()), PCWSTR::null(), PCWSTR::null(), SW_SHOWNORMAL);
                        changed = false;
                    }
                    ID_TRAY_CHOOSE_IMAGE => {
                        if let Some(path) = open_image_dialog(hwnd) {
                            cfg.image_path = Some(path);
                            cfg.style = CrosshairStyle::Image;
                        } else {
                            changed = false;
                        }
                    }
                    ID_TRAY_USE_LOGO => {
                        let logo_path = CrosshairConfig::custom_crosshairs_dir().join("logo.png");
                        cfg.image_path = Some(logo_path.to_string_lossy().to_string());
                        cfg.style = CrosshairStyle::Image;
                    }
                    ID_TRAY_STYLE_CROSS => cfg.style = CrosshairStyle::Cross,
                    ID_TRAY_STYLE_CIRCLE => cfg.style = CrosshairStyle::Circle,
                    ID_TRAY_STYLE_DOT => cfg.style = CrosshairStyle::Dot,
                    ID_TRAY_STYLE_IMAGE => {
                        cfg.style = CrosshairStyle::Image;
                        if cfg.image_path.is_none() {
                            let logo_path = CrosshairConfig::custom_crosshairs_dir().join("logo.png");
                            cfg.image_path = Some(logo_path.to_string_lossy().to_string());
                        }
                    }
                    ID_TRAY_SIZE_MICRO => cfg.size = 1,
                    ID_TRAY_SIZE_TINY => cfg.size = 3,
                    ID_TRAY_SIZE_PRO => cfg.size = 6,
                    ID_TRAY_SIZE_MEDIUM => cfg.size = 14,
                    ID_TRAY_SIZE_LARGE => cfg.size = 30,
                    ID_TRAY_SIZE_XL => cfg.size = 75,
                    ID_TRAY_SIZE_MASSIVE => cfg.size = 150,
                    ID_TRAY_COLOR_GREEN => cfg.color = Color::GREEN,
                    ID_TRAY_COLOR_RED => cfg.color = Color::RED,
                    ID_TRAY_COLOR_CYAN => cfg.color = Color::CYAN,
                    ID_TRAY_COLOR_YELLOW => cfg.color = Color::YELLOW,
                    ID_TRAY_COLOR_WHITE => cfg.color = Color::WHITE,
                    ID_TRAY_TOGGLE_OUTLINE => cfg.outline = !cfg.outline,
                    ID_TRAY_TOGGLE_DOT => cfg.dot = !cfg.dot,
                    ID_TRAY_RESET_DEFAULT => {
                        cfg = CrosshairConfig::default();
                    }
                    ID_TRAY_EXIT => {
                        let _ = PostMessageW(hwnd, WM_CLOSE, WPARAM(0), LPARAM(0));
                        return LRESULT(0);
                    }
                    _ => {
                        changed = false;
                    }
                }
            }

            if changed {
                set_config(cfg);
                notify_overlay_update();
                notify_settings_update();
            }

            LRESULT(0)
        }

        WM_DESTROY => {
            remove_tray_icon(hwnd);
            let _ = get_config().save();
            PostQuitMessage(0);
            LRESULT(0)
        }

        _ => DefWindowProcW(hwnd, msg, wparam, lparam),
    }
}

fn main() -> Result<()> {
    unsafe {
        let hinstance: HINSTANCE = GetModuleHandleW(None)?.into();
        APP_INSTANCE.with(|i| *i.borrow_mut() = hinstance);

        let app_icon = create_logo_icon().unwrap_or_default();
        let class_name = to_wide("MattsCrosshairOverlayClass");
        let wc = WNDCLASSEXW {
            cbSize: mem::size_of::<WNDCLASSEXW>() as u32,
            style: CS_HREDRAW | CS_VREDRAW,
            lpfnWndProc: Some(overlay_wnd_proc),
            hInstance: hinstance,
            hIcon: app_icon,
            hIconSm: app_icon,
            hCursor: LoadCursorW(None, IDC_ARROW)?,
            hbrBackground: HBRUSH(GetStockObject(NULL_BRUSH).0),
            lpszClassName: PCWSTR(class_name.as_ptr()),
            ..Default::default()
        };
        RegisterClassExW(&wc);

        let cfg = get_config();
        let (cx, cy) = monitor::get_monitor_center(cfg.monitor_index);
        let overlay_size = OVERLAY_CANVAS_SIZE;
        let x = cx - overlay_size / 2;
        let y = cy - overlay_size / 2;

        let title = to_wide("Matt's Crosshair");
        let hwnd = CreateWindowExW(
            WS_EX_LAYERED | WS_EX_TRANSPARENT | WS_EX_TOPMOST | WS_EX_TOOLWINDOW | WS_EX_NOACTIVATE,
            PCWSTR(class_name.as_ptr()),
            PCWSTR(title.as_ptr()),
            WS_POPUP | WS_VISIBLE,
            x,
            y,
            overlay_size,
            overlay_size,
            None,
            None,
            hinstance,
            None,
        )?;

        let _ = ShowWindow(hwnd, SW_SHOW);

        let mut msg = MSG::default();
        loop {
            let ret = GetMessageW(&mut msg, None, 0, 0);
            if ret.0 == 0 || ret.0 == -1 {
                break;
            }
            let _ = TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }

        Ok(())
    }
}
