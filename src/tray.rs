use windows::Win32::UI::WindowsAndMessaging::*;
use windows::Win32::UI::Shell::*;
use windows::Win32::Foundation::*;
use windows::core::PCWSTR;
use std::mem;

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

    let tip = to_wide("Matt's Crosshair (Right-click: Menu, Double-click: Settings)");
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

    // Monitors submenu
    let monitors = crate::monitor::enumerate_monitors();
    for (i, mon) in monitors.iter().enumerate() {
        let mut flags = MF_STRING;
        if i == cfg.monitor_index {
            flags |= MF_CHECKED;
        }
        let item_id = (ID_TRAY_MONITOR_BASE + i as u32) as usize;
        AppendMenuW(hmonitors, flags, item_id, PCWSTR(to_wide(&mon.name).as_ptr())).unwrap();
    }

    // Style submenu
    AppendMenuW(hstyle, MF_STRING, ID_TRAY_STYLE_CROSS as usize, PCWSTR(to_wide("Cross").as_ptr())).unwrap();
    AppendMenuW(hstyle, MF_STRING, ID_TRAY_STYLE_CIRCLE as usize, PCWSTR(to_wide("Circle").as_ptr())).unwrap();
    AppendMenuW(hstyle, MF_STRING, ID_TRAY_STYLE_DOT as usize, PCWSTR(to_wide("Dot (Pro)").as_ptr())).unwrap();
    AppendMenuW(hstyle, MF_STRING, ID_TRAY_STYLE_IMAGE as usize, PCWSTR(to_wide("Custom Image").as_ptr())).unwrap();

    // Quick Size submenu (1 to 200 px options)
    AppendMenuW(hsize, MF_STRING, ID_TRAY_SIZE_MICRO as usize, PCWSTR(to_wide("Micro (1 px - Competitive)").as_ptr())).unwrap();
    AppendMenuW(hsize, MF_STRING, ID_TRAY_SIZE_TINY as usize, PCWSTR(to_wide("Tiny (3 px)").as_ptr())).unwrap();
    AppendMenuW(hsize, MF_STRING, ID_TRAY_SIZE_PRO as usize, PCWSTR(to_wide("Pro (6 px)").as_ptr())).unwrap();
    AppendMenuW(hsize, MF_STRING, ID_TRAY_SIZE_MEDIUM as usize, PCWSTR(to_wide("Medium (14 px)").as_ptr())).unwrap();
    AppendMenuW(hsize, MF_STRING, ID_TRAY_SIZE_LARGE as usize, PCWSTR(to_wide("Large (30 px)").as_ptr())).unwrap();
    AppendMenuW(hsize, MF_STRING, ID_TRAY_SIZE_XL as usize, PCWSTR(to_wide("Extra Large (75 px)").as_ptr())).unwrap();
    AppendMenuW(hsize, MF_STRING, ID_TRAY_SIZE_MASSIVE as usize, PCWSTR(to_wide("Massive (150 px)").as_ptr())).unwrap();

    // Color submenu
    AppendMenuW(hcolor, MF_STRING, ID_TRAY_COLOR_GREEN as usize, PCWSTR(to_wide("Green").as_ptr())).unwrap();
    AppendMenuW(hcolor, MF_STRING, ID_TRAY_COLOR_RED as usize, PCWSTR(to_wide("Red").as_ptr())).unwrap();
    AppendMenuW(hcolor, MF_STRING, ID_TRAY_COLOR_CYAN as usize, PCWSTR(to_wide("Cyan").as_ptr())).unwrap();
    AppendMenuW(hcolor, MF_STRING, ID_TRAY_COLOR_YELLOW as usize, PCWSTR(to_wide("Yellow").as_ptr())).unwrap();
    AppendMenuW(hcolor, MF_STRING, ID_TRAY_COLOR_WHITE as usize, PCWSTR(to_wide("White").as_ptr())).unwrap();

    // Main context menu
    AppendMenuW(hmenu, MF_STRING, ID_TRAY_SETTINGS as usize, PCWSTR(to_wide("⚙ Settings & Sliders...").as_ptr())).unwrap();
    AppendMenuW(hmenu, MF_STRING, ID_TRAY_OPEN_FOLDER as usize, PCWSTR(to_wide("📂 Open Crosshairs Folder...").as_ptr())).unwrap();
    AppendMenuW(hmenu, MF_STRING, ID_TRAY_USE_LOGO as usize, PCWSTR(to_wide("🎯 Set Red Logo Crosshair").as_ptr())).unwrap();
    AppendMenuW(hmenu, MF_SEPARATOR, 0, PCWSTR::null()).unwrap();

    AppendMenuW(hmenu, MF_STRING | MF_POPUP, hmonitors.0 as usize, PCWSTR(to_wide("Monitor / Display").as_ptr())).unwrap();
    AppendMenuW(hmenu, MF_STRING | MF_POPUP, hstyle.0 as usize, PCWSTR(to_wide("Style").as_ptr())).unwrap();
    AppendMenuW(hmenu, MF_STRING | MF_POPUP, hsize.0 as usize, PCWSTR(to_wide("Quick Size").as_ptr())).unwrap();
    AppendMenuW(hmenu, MF_STRING | MF_POPUP, hcolor.0 as usize, PCWSTR(to_wide("Color").as_ptr())).unwrap();

    AppendMenuW(hmenu, MF_SEPARATOR, 0, PCWSTR::null()).unwrap();
    AppendMenuW(hmenu, MF_STRING, ID_TRAY_TOGGLE_OUTLINE as usize, PCWSTR(to_wide("Toggle Black Outline").as_ptr())).unwrap();
    AppendMenuW(hmenu, MF_STRING, ID_TRAY_TOGGLE_DOT as usize, PCWSTR(to_wide("Toggle Center Dot").as_ptr())).unwrap();
    AppendMenuW(hmenu, MF_SEPARATOR, 0, PCWSTR::null()).unwrap();
    AppendMenuW(hmenu, MF_STRING, ID_TRAY_RESET_DEFAULT as usize, PCWSTR(to_wide("Reset to Default").as_ptr())).unwrap();

    // Shutdown with remote power off symbol ⏻
    AppendMenuW(hmenu, MF_STRING, ID_TRAY_EXIT as usize, PCWSTR(to_wide("⏻ Shutdown").as_ptr())).unwrap();

    let mut pt = POINT::default();
    let _ = GetCursorPos(&mut pt);
    let _ = SetForegroundWindow(hwnd);
    let _ = TrackPopupMenu(hmenu, TPM_BOTTOMALIGN | TPM_LEFTALIGN, pt.x, pt.y, 0, hwnd, None);
    let _ = DestroyMenu(hmenu);
}
