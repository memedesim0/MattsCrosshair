use windows::Win32::UI::WindowsAndMessaging::*;
use windows::Win32::UI::Shell::*;
use windows::Win32::Foundation::*;
use windows::core::PCWSTR;
use std::mem;
use crate::config::{Color, CrosshairStyle};

pub const WM_TRAY: u32 = WM_USER + 1;

pub const ID_TRAY_SETTINGS: u32 = 1000;
pub const ID_TRAY_EXIT: u32 = 1001;

pub const ID_TRAY_COLOR_GREEN: u32 = 1002;
pub const ID_TRAY_COLOR_RED: u32 = 1003;
pub const ID_TRAY_COLOR_CYAN: u32 = 1004;
pub const ID_TRAY_COLOR_YELLOW: u32 = 1005;
pub const ID_TRAY_COLOR_WHITE: u32 = 1006;

pub const ID_TRAY_SIZE_MICRO: u32 = 1010;
pub const ID_TRAY_SIZE_TINY: u32 = 1011;
pub const ID_TRAY_SIZE_PRO: u32 = 1012;
pub const ID_TRAY_SIZE_MEDIUM: u32 = 1013;
pub const ID_TRAY_SIZE_LARGE: u32 = 1014;
pub const ID_TRAY_SIZE_XL: u32 = 1015;
pub const ID_TRAY_SIZE_MASSIVE: u32 = 1016;

pub const ID_TRAY_STYLE_CROSS: u32 = 1020;
pub const ID_TRAY_STYLE_CIRCLE: u32 = 1021;
pub const ID_TRAY_STYLE_DOT: u32 = 1022;
pub const ID_TRAY_STYLE_IMAGE: u32 = 1023;

pub const ID_TRAY_CHOOSE_IMAGE: u32 = 1025;
pub const ID_TRAY_USE_LOGO: u32 = 1026;
pub const ID_TRAY_OPEN_FOLDER: u32 = 1027;
pub const ID_TRAY_TOGGLE_OUTLINE: u32 = 1030;
pub const ID_TRAY_TOGGLE_DOT: u32 = 1031;

pub const ID_TRAY_RESET_DEFAULT: u32 = 1050;

pub const ID_TRAY_MONITOR_BASE: u32 = 1100;

fn to_wide(s: &str) -> Vec<u16> {
    s.encode_utf16().chain(std::iter::once(0)).collect()
}

pub unsafe fn add_tray_icon(hwnd: HWND) {
    let mut nid: NOTIFYICONDATAW = mem::zeroed();
    nid.cbSize = mem::size_of::<NOTIFYICONDATAW>() as u32;
    nid.hWnd = hwnd;
    nid.uID = 1;
    nid.uFlags = NIF_ICON | NIF_MESSAGE | NIF_TIP;
    nid.uCallbackMessage = WM_TRAY;

    // Custom thick red logo with white outline
    nid.hIcon = crate::create_logo_icon().unwrap_or_else(|| LoadIconW(None, IDI_APPLICATION).unwrap_or_default());

    let tip = to_wide("Matt's Crosshair (Tasto destro: Menu, Doppio clic: Impostazioni)");
    let len = tip.len().min(nid.szTip.len());
    nid.szTip[..len].copy_from_slice(&tip[..len]);
    let _ = Shell_NotifyIconW(NIM_ADD, &nid);
}

pub unsafe fn remove_tray_icon(hwnd: HWND) {
    let mut nid: NOTIFYICONDATAW = mem::zeroed();
    nid.cbSize = mem::size_of::<NOTIFYICONDATAW>() as u32;
    nid.hWnd = hwnd;
    nid.uID = 1;
    let _ = Shell_NotifyIconW(NIM_DELETE, &nid);
}

pub unsafe fn show_context_menu(hwnd: HWND) {
    let hmenu = CreatePopupMenu().unwrap();

    let hstyle = CreatePopupMenu().unwrap();
    let hsize = CreatePopupMenu().unwrap();
    let hcolor = CreatePopupMenu().unwrap();
    let hmonitors = CreatePopupMenu().unwrap();

    let cfg = crate::get_config();

    // Sottomenu Monitor / Schermi
    let monitors = crate::monitor::enumerate_monitors();
    for (i, mon) in monitors.iter().enumerate() {
        let mut flags = MF_STRING;
        if i == cfg.monitor_index {
            flags |= MF_CHECKED;
        }
        let item_id = (ID_TRAY_MONITOR_BASE + i as u32) as usize;
        AppendMenuW(hmonitors, flags, item_id, PCWSTR(to_wide(&mon.name).as_ptr())).unwrap();
    }

    // Sottomenu Stile / Forma
    let check_cross = if cfg.style == CrosshairStyle::Cross { MF_CHECKED } else { MF_UNCHECKED };
    let check_circle = if cfg.style == CrosshairStyle::Circle { MF_CHECKED } else { MF_UNCHECKED };
    let check_dot = if cfg.style == CrosshairStyle::Dot { MF_CHECKED } else { MF_UNCHECKED };
    let check_img = if cfg.style == CrosshairStyle::Image { MF_CHECKED } else { MF_UNCHECKED };

    AppendMenuW(hstyle, MF_STRING | check_cross, ID_TRAY_STYLE_CROSS as usize, PCWSTR(to_wide("Croce (Cross)").as_ptr())).unwrap();
    AppendMenuW(hstyle, MF_STRING | check_circle, ID_TRAY_STYLE_CIRCLE as usize, PCWSTR(to_wide("Cerchio (Circle)").as_ptr())).unwrap();
    AppendMenuW(hstyle, MF_STRING | check_dot, ID_TRAY_STYLE_DOT as usize, PCWSTR(to_wide("Solo Punto (Dot)").as_ptr())).unwrap();
    AppendMenuW(hstyle, MF_STRING | check_img, ID_TRAY_STYLE_IMAGE as usize, PCWSTR(to_wide("Immagine Personalizzata (Custom Image)").as_ptr())).unwrap();

    // Sottomenu Dimensioni rapide (1 a 150 px)
    AppendMenuW(hsize, MF_STRING, ID_TRAY_SIZE_MICRO as usize, PCWSTR(to_wide("Micro (1 px)").as_ptr())).unwrap();
    AppendMenuW(hsize, MF_STRING, ID_TRAY_SIZE_TINY as usize, PCWSTR(to_wide("Minuscolo (3 px)").as_ptr())).unwrap();
    AppendMenuW(hsize, MF_STRING, ID_TRAY_SIZE_PRO as usize, PCWSTR(to_wide("Pro (6 px)").as_ptr())).unwrap();
    AppendMenuW(hsize, MF_STRING, ID_TRAY_SIZE_MEDIUM as usize, PCWSTR(to_wide("Medio (14 px)").as_ptr())).unwrap();
    AppendMenuW(hsize, MF_STRING, ID_TRAY_SIZE_LARGE as usize, PCWSTR(to_wide("Grande (30 px)").as_ptr())).unwrap();
    AppendMenuW(hsize, MF_STRING, ID_TRAY_SIZE_XL as usize, PCWSTR(to_wide("Molto grande (75 px)").as_ptr())).unwrap();
    AppendMenuW(hsize, MF_STRING, ID_TRAY_SIZE_MASSIVE as usize, PCWSTR(to_wide("Massivo (150 px)").as_ptr())).unwrap();

    // Sottomenu Colore
    let check_g = if cfg.color == Color::GREEN { MF_CHECKED } else { MF_UNCHECKED };
    let check_r = if cfg.color == Color::RED { MF_CHECKED } else { MF_UNCHECKED };
    let check_c = if cfg.color == Color::CYAN { MF_CHECKED } else { MF_UNCHECKED };
    let check_y = if cfg.color == Color::YELLOW { MF_CHECKED } else { MF_UNCHECKED };
    let check_w = if cfg.color == Color::WHITE { MF_CHECKED } else { MF_UNCHECKED };

    AppendMenuW(hcolor, MF_STRING | check_g, ID_TRAY_COLOR_GREEN as usize, PCWSTR(to_wide("Verde").as_ptr())).unwrap();
    AppendMenuW(hcolor, MF_STRING | check_r, ID_TRAY_COLOR_RED as usize, PCWSTR(to_wide("Rosso").as_ptr())).unwrap();
    AppendMenuW(hcolor, MF_STRING | check_c, ID_TRAY_COLOR_CYAN as usize, PCWSTR(to_wide("Ciano").as_ptr())).unwrap();
    AppendMenuW(hcolor, MF_STRING | check_y, ID_TRAY_COLOR_YELLOW as usize, PCWSTR(to_wide("Giallo").as_ptr())).unwrap();
    AppendMenuW(hcolor, MF_STRING | check_w, ID_TRAY_COLOR_WHITE as usize, PCWSTR(to_wide("Bianco").as_ptr())).unwrap();

    // Menu contestuale principale
    AppendMenuW(hmenu, MF_STRING, ID_TRAY_SETTINGS as usize, PCWSTR(to_wide("⚙ Impostazioni Mirino & Slider...").as_ptr())).unwrap();
    AppendMenuW(hmenu, MF_STRING, ID_TRAY_OPEN_FOLDER as usize, PCWSTR(to_wide("📂 Apri Cartella Mirini...").as_ptr())).unwrap();
    AppendMenuW(hmenu, MF_STRING, ID_TRAY_USE_LOGO as usize, PCWSTR(to_wide("🎯 Imposta Logo Rosso Originale").as_ptr())).unwrap();
    AppendMenuW(hmenu, MF_SEPARATOR, 0, PCWSTR::null()).unwrap();

    AppendMenuW(hmenu, MF_STRING | MF_POPUP, hmonitors.0 as usize, PCWSTR(to_wide("📺 Schermo / Monitor").as_ptr())).unwrap();
    AppendMenuW(hmenu, MF_STRING | MF_POPUP, hstyle.0 as usize, PCWSTR(to_wide("Forma / Stile").as_ptr())).unwrap();
    AppendMenuW(hmenu, MF_STRING | MF_POPUP, hsize.0 as usize, PCWSTR(to_wide("Dimensioni Rapide").as_ptr())).unwrap();
    AppendMenuW(hmenu, MF_STRING | MF_POPUP, hcolor.0 as usize, PCWSTR(to_wide("Colore").as_ptr())).unwrap();

    AppendMenuW(hmenu, MF_SEPARATOR, 0, PCWSTR::null()).unwrap();
    let chk_out = if cfg.outline { MF_CHECKED } else { MF_UNCHECKED };
    let chk_dt = if cfg.dot { MF_CHECKED } else { MF_UNCHECKED };
    AppendMenuW(hmenu, MF_STRING | chk_out, ID_TRAY_TOGGLE_OUTLINE as usize, PCWSTR(to_wide("Bordo Nero (Outline)").as_ptr())).unwrap();
    AppendMenuW(hmenu, MF_STRING | chk_dt, ID_TRAY_TOGGLE_DOT as usize, PCWSTR(to_wide("Punto Centrale (Dot)").as_ptr())).unwrap();
    AppendMenuW(hmenu, MF_SEPARATOR, 0, PCWSTR::null()).unwrap();
    AppendMenuW(hmenu, MF_STRING, ID_TRAY_RESET_DEFAULT as usize, PCWSTR(to_wide("Ripristina Predefiniti").as_ptr())).unwrap();

    // Shutdown con icona ⏻
    AppendMenuW(hmenu, MF_STRING, ID_TRAY_EXIT as usize, PCWSTR(to_wide("⏻ Chiudi Matt's Crosshair").as_ptr())).unwrap();

    let mut pt = POINT::default();
    let _ = GetCursorPos(&mut pt);
    let _ = SetForegroundWindow(hwnd);
    let _ = TrackPopupMenu(hmenu, TPM_BOTTOMALIGN | TPM_LEFTALIGN, pt.x, pt.y, 0, hwnd, None);
    let _ = DestroyMenu(hmenu);
}
