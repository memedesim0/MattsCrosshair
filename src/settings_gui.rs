use crate::config::{Color, CrosshairConfig, CrosshairStyle};
use crate::monitor::enumerate_monitors;
use std::cell::RefCell;
use std::mem;
use std::path::Path;
use windows::{
    core::*,
    Win32::Foundation::*,
    Win32::Graphics::Gdi::*,
    Win32::System::LibraryLoader::GetModuleHandleW,
    Win32::UI::Controls::Dialogs::*,
    Win32::UI::Controls::*,
    Win32::UI::Shell::ShellExecuteW,
    Win32::UI::WindowsAndMessaging::*,
};

extern "system" {
    fn EnableWindow(hwnd: HWND, benable: BOOL) -> BOOL;
}

pub const WM_APP_UPDATE: u32 = WM_USER + 50;

// Control IDs
const ID_COMBO_MONITOR: u32 = 1999;

// Top-level Mode Selector
const ID_MODE_STANDARD: u32 = 2000;
const ID_MODE_IMAGE: u32 = 2004;

// Standard Shapes
const ID_RADIO_CROSS: u32 = 2001;
const ID_RADIO_CIRCLE: u32 = 2002;
const ID_RADIO_DOT: u32 = 2003;

// Sliders
const ID_SLIDER_SIZE: u32 = 2010;
const ID_SLIDER_THICKNESS: u32 = 2011;
const ID_SLIDER_GAP: u32 = 2012;
const ID_SLIDER_DOT_SIZE: u32 = 2013;
const ID_SLIDER_OPACITY: u32 = 2014;

// Checkboxes
const ID_CHECK_OUTLINE: u32 = 2020;
const ID_CHECK_DOT: u32 = 2021;

// Custom Image Library
const ID_LIST_CUSTOM: u32 = 2030;
const ID_BTN_IMPORT: u32 = 2031;
const ID_BTN_OPEN_FOLDER: u32 = 2032;
const ID_BTN_DELETE: u32 = 2033;

// Color Presets (Top section for Standard mode)
const ID_BTN_COLOR_GREEN: u32 = 2040;
const ID_BTN_COLOR_RED: u32 = 2041;
const ID_BTN_COLOR_CYAN: u32 = 2042;
const ID_BTN_COLOR_YELLOW: u32 = 2043;
const ID_BTN_COLOR_WHITE: u32 = 2044;
const ID_BTN_COLOR_PICKER: u32 = 2045;

const TBM_GETPOS: u32 = WM_USER;
const TBM_SETPOS: u32 = WM_USER + 5;
const TBM_SETRANGE: u32 = WM_USER + 6;
const BM_GETCHECK: u32 = 0x00F0;
const BM_SETCHECK: u32 = 0x00F1;
const BST_CHECKED: usize = 1;
const BST_UNCHECKED: usize = 0;

const LB_ADDSTRING: u32 = 0x0180;
const LB_RESETCONTENT: u32 = 0x0184;
const LB_SETCURSEL: u32 = 0x0186;
const LB_GETCURSEL: u32 = 0x0188;
const LB_GETTEXT: u32 = 0x0189;
const LBN_SELCHANGE: u16 = 1;

const CB_ADDSTRING: u32 = 0x0143;
const CB_SETCURSEL: u32 = 0x014E;
const CB_GETCURSEL: u32 = 0x0147;
const CB_RESETCONTENT: u32 = 0x014B;
const CBN_SELCHANGE: u16 = 1;

thread_local! {
    static OVERLAY_HWND: RefCell<HWND> = RefCell::new(HWND(std::ptr::null_mut()));
    static SETTINGS_HWND: RefCell<HWND> = RefCell::new(HWND(std::ptr::null_mut()));

    // Target Monitor
    static LBL_MONITOR_HEADER: RefCell<HWND> = RefCell::new(HWND(std::ptr::null_mut()));
    static COMBO_MONITOR: RefCell<HWND> = RefCell::new(HWND(std::ptr::null_mut()));
    static LBL_MONITOR_SUB: RefCell<HWND> = RefCell::new(HWND(std::ptr::null_mut()));

    // Mode Selector (Standard vs Custom Image)
    static LBL_MODE_HEADER: RefCell<HWND> = RefCell::new(HWND(std::ptr::null_mut()));
    static RADIO_MODE_STANDARD: RefCell<HWND> = RefCell::new(HWND(std::ptr::null_mut()));
    static RADIO_MODE_IMAGE: RefCell<HWND> = RefCell::new(HWND(std::ptr::null_mut()));

    // Standard mode only: Color (at the top!)
    static LBL_COLOR_HEADER: RefCell<HWND> = RefCell::new(HWND(std::ptr::null_mut()));
    static BTN_COLORS: RefCell<Vec<HWND>> = RefCell::new(Vec::new());
    static LBL_COLOR_CURRENT: RefCell<HWND> = RefCell::new(HWND(std::ptr::null_mut()));

    // Standard mode only: Shapes
    static LBL_SHAPE_HEADER: RefCell<HWND> = RefCell::new(HWND(std::ptr::null_mut()));
    static RADIO_CROSS: RefCell<HWND> = RefCell::new(HWND(std::ptr::null_mut()));
    static RADIO_CIRCLE: RefCell<HWND> = RefCell::new(HWND(std::ptr::null_mut()));
    static RADIO_DOT: RefCell<HWND> = RefCell::new(HWND(std::ptr::null_mut()));

    // Standard mode only: Thickness, Gap, Outline, Center Dot
    static LBL_THICKNESS: RefCell<HWND> = RefCell::new(HWND(std::ptr::null_mut()));
    static SLIDER_THICKNESS: RefCell<HWND> = RefCell::new(HWND(std::ptr::null_mut()));
    static LBL_GAP: RefCell<HWND> = RefCell::new(HWND(std::ptr::null_mut()));
    static SLIDER_GAP: RefCell<HWND> = RefCell::new(HWND(std::ptr::null_mut()));
    static CHK_OUTLINE: RefCell<HWND> = RefCell::new(HWND(std::ptr::null_mut()));
    static CHK_DOT: RefCell<HWND> = RefCell::new(HWND(std::ptr::null_mut()));
    static LBL_DOT_SIZE: RefCell<HWND> = RefCell::new(HWND(std::ptr::null_mut()));
    static SLIDER_DOT_SIZE: RefCell<HWND> = RefCell::new(HWND(std::ptr::null_mut()));

    // Custom Image mode only
    static LBL_CUSTOM_HEADER: RefCell<HWND> = RefCell::new(HWND(std::ptr::null_mut()));
    static BTN_IMPORT_H: RefCell<HWND> = RefCell::new(HWND(std::ptr::null_mut()));
    static BTN_OPEN_FOLDER_H: RefCell<HWND> = RefCell::new(HWND(std::ptr::null_mut()));
    static BTN_DELETE_H: RefCell<HWND> = RefCell::new(HWND(std::ptr::null_mut()));
    static LIST_CUSTOM: RefCell<HWND> = RefCell::new(HWND(std::ptr::null_mut()));
    static LBL_ACTIVE_IMAGE: RefCell<HWND> = RefCell::new(HWND(std::ptr::null_mut()));

    // Common / repositioned controls
    static LBL_SIZE: RefCell<HWND> = RefCell::new(HWND(std::ptr::null_mut()));
    static SLIDER_SIZE: RefCell<HWND> = RefCell::new(HWND(std::ptr::null_mut()));
    static LBL_OPACITY: RefCell<HWND> = RefCell::new(HWND(std::ptr::null_mut()));
    static SLIDER_OPACITY: RefCell<HWND> = RefCell::new(HWND(std::ptr::null_mut()));
    static LBL_TIP: RefCell<HWND> = RefCell::new(HWND(std::ptr::null_mut()));
}

fn to_wide(s: &str) -> Vec<u16> {
    s.encode_utf16().chain(std::iter::once(0)).collect()
}

pub fn set_overlay_hwnd(hwnd: HWND) {
    OVERLAY_HWND.with(|h| *h.borrow_mut() = hwnd);
}

pub fn notify_overlay_update() {
    OVERLAY_HWND.with(|h| {
        let hwnd = *h.borrow();
        if !hwnd.0.is_null() {
            unsafe {
                let _ = PostMessageW(hwnd, WM_APP_UPDATE, WPARAM(0), LPARAM(0));
            }
        }
    });
}

pub fn notify_settings_update() {
    SETTINGS_HWND.with(|s| {
        let hwnd = *s.borrow();
        if !hwnd.0.is_null() {
            unsafe {
                let _ = PostMessageW(hwnd, WM_APP_UPDATE, WPARAM(0), LPARAM(0));
            }
        }
    });
}

pub fn show_settings_window(hinstance: HINSTANCE) {
    SETTINGS_HWND.with(|s| {
        let hwnd = *s.borrow();
        if !hwnd.0.is_null() {
            unsafe {
                let cfg = crate::get_config();
                refresh_custom_list();
                refresh_monitors_list();
                update_ui_layout_and_visibility(hwnd, &cfg);
                let _ = ShowWindow(hwnd, SW_SHOW);
                let _ = SetForegroundWindow(hwnd);
            }
        } else {
            create_settings_window(hinstance);
        }
    });
}

fn create_settings_window(hinstance: HINSTANCE) {
    unsafe {
        let icex = INITCOMMONCONTROLSEX {
            dwSize: mem::size_of::<INITCOMMONCONTROLSEX>() as u32,
            dwICC: ICC_BAR_CLASSES,
        };
        let _ = InitCommonControlsEx(&icex);

        let class_name = to_wide("MattsCrosshairSettingsClass");
        let app_icon = crate::create_logo_icon().unwrap_or_default();
        let wc = WNDCLASSEXW {
            cbSize: mem::size_of::<WNDCLASSEXW>() as u32,
            style: CS_HREDRAW | CS_VREDRAW,
            lpfnWndProc: Some(settings_wnd_proc),
            hInstance: hinstance,
            hIcon: app_icon,
            hIconSm: app_icon,
            hCursor: LoadCursorW(None, IDC_ARROW).unwrap_or_default(),
            hbrBackground: HBRUSH((COLOR_WINDOW.0 + 1) as *mut std::ffi::c_void),
            lpszClassName: PCWSTR(class_name.as_ptr()),
            ..Default::default()
        };
        RegisterClassExW(&wc);

        let title = to_wide("Matt's Crosshair — Impostazioni & Personalizzazione");
        let hwnd = CreateWindowExW(
            WINDOW_EX_STYLE::default(),
            PCWSTR(class_name.as_ptr()),
            PCWSTR(title.as_ptr()),
            WS_OVERLAPPED | WS_CAPTION | WS_SYSMENU | WS_MINIMIZEBOX,
            CW_USEDEFAULT,
            CW_USEDEFAULT,
            490,
            610,
            None,
            None,
            hinstance,
            None,
        )
        .unwrap();

        SETTINGS_HWND.with(|s| *s.borrow_mut() = hwnd);
        let _ = ShowWindow(hwnd, SW_SHOW);
        let _ = UpdateWindow(hwnd);
    }
}

unsafe fn create_controls(hwnd: HWND, hinstance: HINSTANCE) {
    let cfg = crate::get_config();
    let font = GetStockObject(DEFAULT_GUI_FONT);

    // =========================================================================
    // 1. MONITOR SELECTION (In alto, sempre visibile)
    // =========================================================================
    let lbl_mon = create_label(hwnd, hinstance, "SCHERMO DI VISUALIZZAZIONE:", 20, 10, 430, 16, font);
    LBL_MONITOR_HEADER.with(|l| *l.borrow_mut() = lbl_mon);

    let cls_cb = to_wide("COMBOBOX");
    let h_cb = CreateWindowExW(
        WINDOW_EX_STYLE::default(),
        PCWSTR(cls_cb.as_ptr()),
        PCWSTR::null(),
        WS_CHILD | WS_VISIBLE | WS_TABSTOP | WS_VSCROLL | WINDOW_STYLE(CBS_DROPDOWNLIST as u32),
        20,
        28,
        430,
        200,
        hwnd,
        HMENU(ID_COMBO_MONITOR as *mut std::ffi::c_void),
        hinstance,
        None,
    )
    .unwrap();
    SendMessageW(h_cb, WM_SETFONT, WPARAM(font.0 as usize), LPARAM(1));
    COMBO_MONITOR.with(|c| *c.borrow_mut() = h_cb);

    let lbl_mon_sub = create_label(
        hwnd,
        hinstance,
        "Il mirino compare esattamente al centro dello schermo selezionato.",
        20,
        56,
        430,
        14,
        font,
    );
    LBL_MONITOR_SUB.with(|l| *l.borrow_mut() = lbl_mon_sub);

    // =========================================================================
    // 2. SELETTORE MODALITÀ (Standard Crosshair vs Custom Image)
    // =========================================================================
    let lbl_mode = create_label(hwnd, hinstance, "MODALITÀ DEL MIRINO:", 20, 76, 430, 16, font);
    LBL_MODE_HEADER.with(|l| *l.borrow_mut() = lbl_mode);

    let r_std = create_radio(hwnd, hinstance, "Mirino Standard (Forme)", 20, 94, 200, 24, ID_MODE_STANDARD, font);
    let r_img = create_radio(hwnd, hinstance, "Immagine Personalizzata", 230, 94, 220, 24, ID_MODE_IMAGE, font);
    RADIO_MODE_STANDARD.with(|h| *h.borrow_mut() = r_std);
    RADIO_MODE_IMAGE.with(|h| *h.borrow_mut() = r_img);

    // =========================================================================
    // 3. CONTROLLI ESCLUSIVI PER MIRINO STANDARD
    // =========================================================================

    // --- COLORE DEL MIRINO (PORTATO IN ALTO COME RICHIESTO) ---
    let lbl_col = create_label(hwnd, hinstance, "COLORE DEL MIRINO:", 20, 126, 430, 16, font);
    LBL_COLOR_HEADER.with(|l| *l.borrow_mut() = lbl_col);

    let btn_g = create_button(hwnd, hinstance, "Verde", 20, 146, 62, 25, ID_BTN_COLOR_GREEN, font);
    let btn_r = create_button(hwnd, hinstance, "Rosso", 88, 146, 62, 25, ID_BTN_COLOR_RED, font);
    let btn_c = create_button(hwnd, hinstance, "Ciano", 156, 146, 62, 25, ID_BTN_COLOR_CYAN, font);
    let btn_y = create_button(hwnd, hinstance, "Giallo", 224, 146, 62, 25, ID_BTN_COLOR_YELLOW, font);
    let btn_w = create_button(hwnd, hinstance, "Bianco", 292, 146, 62, 25, ID_BTN_COLOR_WHITE, font);
    let btn_p = create_button(hwnd, hinstance, "🎨 Altro...", 360, 146, 90, 25, ID_BTN_COLOR_PICKER, font);
    BTN_COLORS.with(|b| {
        *b.borrow_mut() = vec![btn_g, btn_r, btn_c, btn_y, btn_w, btn_p];
    });

    let lbl_col_cur = create_label(hwnd, hinstance, "Colore attivo: Verde (0, 255, 0)", 20, 176, 430, 16, font);
    LBL_COLOR_CURRENT.with(|l| *l.borrow_mut() = lbl_col_cur);

    // --- FORMA STANDARD (Croce, Cerchio, Punto) ---
    let lbl_shp = create_label(hwnd, hinstance, "FORMA DEL MIRINO:", 20, 196, 430, 16, font);
    LBL_SHAPE_HEADER.with(|l| *l.borrow_mut() = lbl_shp);

    let rc = create_radio(hwnd, hinstance, "Croce (+)", 20, 214, 100, 24, ID_RADIO_CROSS, font);
    let rcirc = create_radio(hwnd, hinstance, "Cerchio (○)", 130, 214, 110, 24, ID_RADIO_CIRCLE, font);
    let rdot = create_radio(hwnd, hinstance, "Solo Punto (•)", 250, 214, 130, 24, ID_RADIO_DOT, font);
    RADIO_CROSS.with(|h| *h.borrow_mut() = rc);
    RADIO_CIRCLE.with(|h| *h.borrow_mut() = rcirc);
    RADIO_DOT.with(|h| *h.borrow_mut() = rdot);

    // --- SPESSORE (Thickness) ---
    let lbl_t = create_label(hwnd, hinstance, &format!("Spessore linee (Thickness): {} px", cfg.thickness), 20, 290, 430, 16, font);
    let sl_t = create_slider(hwnd, hinstance, 20, 308, 430, 26, ID_SLIDER_THICKNESS, 1, 15, cfg.thickness as i32);
    LBL_THICKNESS.with(|h| *h.borrow_mut() = lbl_t);
    SLIDER_THICKNESS.with(|h| *h.borrow_mut() = sl_t);

    // --- SPAZIO CENTRALE (Center Gap) ---
    let lbl_g = create_label(hwnd, hinstance, &format!("Spazio centrale (Center Gap): {} px", cfg.gap), 20, 338, 430, 16, font);
    let sl_g = create_slider(hwnd, hinstance, 20, 356, 430, 26, ID_SLIDER_GAP, 0, 40, cfg.gap as i32);
    LBL_GAP.with(|h| *h.borrow_mut() = lbl_g);
    SLIDER_GAP.with(|h| *h.borrow_mut() = sl_g);

    // --- CHECKBOX BORDO NERO & PUNTO CENTRALE ---
    let chk_out = create_checkbox(hwnd, hinstance, "Bordo nero (Outline)", 20, 388, 180, 22, ID_CHECK_OUTLINE, font);
    let chk_dt = create_checkbox(hwnd, hinstance, "Aggiungi punto al centro", 210, 388, 240, 22, ID_CHECK_DOT, font);
    CHK_OUTLINE.with(|h| *h.borrow_mut() = chk_out);
    CHK_DOT.with(|h| *h.borrow_mut() = chk_dt);

    if cfg.outline {
        SendMessageW(chk_out, BM_SETCHECK, WPARAM(BST_CHECKED), LPARAM(0));
    }
    if cfg.dot {
        SendMessageW(chk_dt, BM_SETCHECK, WPARAM(BST_CHECKED), LPARAM(0));
    }

    // --- DIMENSIONE PUNTO CENTRALE ---
    let lbl_d = create_label(hwnd, hinstance, &format!("Dimensione punto centrale: {} px", cfg.dot_size), 20, 414, 430, 16, font);
    let sl_d = create_slider(hwnd, hinstance, 20, 432, 430, 26, ID_SLIDER_DOT_SIZE, 1, 15, cfg.dot_size as i32);
    LBL_DOT_SIZE.with(|h| *h.borrow_mut() = lbl_d);
    SLIDER_DOT_SIZE.with(|h| *h.borrow_mut() = sl_d);

    // =========================================================================
    // 4. CONTROLLI ESCLUSIVI PER IMMAGINE PERSONALIZZATA (Libreria & File)
    // =========================================================================
    let lbl_cust = create_label(hwnd, hinstance, "LIBRERIA IMMAGINI (PNG, SVG, WEBP, JPG, BMP):", 20, 126, 430, 16, font);
    LBL_CUSTOM_HEADER.with(|l| *l.borrow_mut() = lbl_cust);

    let btn_imp = create_button(hwnd, hinstance, "➕ Importa File...", 20, 146, 135, 26, ID_BTN_IMPORT, font);
    let btn_fol = create_button(hwnd, hinstance, "📂 Apri Cartella", 162, 146, 140, 26, ID_BTN_OPEN_FOLDER, font);
    let btn_del = create_button(hwnd, hinstance, "🗑️ Elimina Immagine", 310, 146, 140, 26, ID_BTN_DELETE, font);
    BTN_IMPORT_H.with(|b| *b.borrow_mut() = btn_imp);
    BTN_OPEN_FOLDER_H.with(|b| *b.borrow_mut() = btn_fol);
    BTN_DELETE_H.with(|b| *b.borrow_mut() = btn_del);

    let cls_lb = to_wide("LISTBOX");
    let h_lb = CreateWindowExW(
        WINDOW_EX_STYLE::default(),
        PCWSTR(cls_lb.as_ptr()),
        PCWSTR::null(),
        WS_CHILD | WS_VISIBLE | WS_VSCROLL | WS_BORDER | WINDOW_STYLE(LBS_NOTIFY as u32),
        20,
        178,
        430,
        100,
        hwnd,
        HMENU(ID_LIST_CUSTOM as *mut std::ffi::c_void),
        hinstance,
        None,
    )
    .unwrap();
    SendMessageW(h_lb, WM_SETFONT, WPARAM(font.0 as usize), LPARAM(1));
    LIST_CUSTOM.with(|l| *l.borrow_mut() = h_lb);

    let active_txt = format!(
        "File attivo: {}",
        cfg.image_path.as_deref().unwrap_or("logo.svg")
    );
    let lbl_act = create_label(hwnd, hinstance, &active_txt, 20, 284, 430, 16, font);
    LBL_ACTIVE_IMAGE.with(|l| *l.borrow_mut() = lbl_act);

    // =========================================================================
    // 5. CONTROLLI CONDIVISI / RIPOSIZIONATI DINAMICAMENTE (Dimensione & Opacità)
    // =========================================================================
    let lbl_s = create_label(hwnd, hinstance, &format!("Dimensione (Size): {} px", cfg.size), 20, 242, 430, 16, font);
    let sl_s = create_slider(hwnd, hinstance, 20, 260, 430, 26, ID_SLIDER_SIZE, 1, 200, cfg.size as i32);
    LBL_SIZE.with(|h| *h.borrow_mut() = lbl_s);
    SLIDER_SIZE.with(|h| *h.borrow_mut() = sl_s);

    let op_pct = ((cfg.opacity as f32 / 255.0) * 100.0).round() as i32;
    let lbl_o = create_label(hwnd, hinstance, &format!("Opacità (Opacity): {}%", op_pct), 20, 462, 430, 16, font);
    let sl_o = create_slider(hwnd, hinstance, 20, 480, 430, 26, ID_SLIDER_OPACITY, 20, 100, op_pct);
    LBL_OPACITY.with(|h| *h.borrow_mut() = lbl_o);
    SLIDER_OPACITY.with(|h| *h.borrow_mut() = sl_o);

    let tip = "Tasto destro sull'icona nella barra di sistema per il menu rapido e Spegnimento.";
    let lbl_tp = create_label(hwnd, hinstance, tip, 20, 516, 430, 28, font);
    LBL_TIP.with(|h| *h.borrow_mut() = lbl_tp);

    // Inizializza liste e stato visivo
    refresh_monitors_list();
    refresh_custom_list();
    update_ui_layout_and_visibility(hwnd, &cfg);
}

unsafe fn show_ctrl(cell: &'static std::thread::LocalKey<RefCell<HWND>>, show: bool) {
    cell.with(|c| {
        let h = *c.borrow();
        if !h.0.is_null() {
            let _ = ShowWindow(h, if show { SW_SHOW } else { SW_HIDE });
        }
    });
}

unsafe fn move_ctrl(cell: &'static std::thread::LocalKey<RefCell<HWND>>, x: i32, y: i32, w: i32, h: i32) {
    cell.with(|c| {
        let hwnd = *c.borrow();
        if !hwnd.0.is_null() {
            let _ = SetWindowPos(hwnd, None, x, y, w, h, SWP_NOZORDER | SWP_NOACTIVATE);
        }
    });
}

unsafe fn update_color_preview_label(color: Color) {
    LBL_COLOR_CURRENT.with(|lbl| {
        let h = *lbl.borrow();
        if !h.0.is_null() {
            let color_name = if color == Color::GREEN {
                "Verde"
            } else if color == Color::RED {
                "Rosso"
            } else if color == Color::CYAN {
                "Ciano"
            } else if color == Color::YELLOW {
                "Giallo"
            } else if color == Color::WHITE {
                "Bianco"
            } else {
                "Personalizzato"
            };
            let text = format!("Colore attivo: {} (R:{} G:{} B:{})", color_name, color.r, color.g, color.b);
            update_label(h, &text);
        }
    });
}

/// Aggiorna la visibilità e il layout dell'interfaccia in base alla modalità selezionata
pub unsafe fn update_ui_layout_and_visibility(hwnd: HWND, cfg: &CrosshairConfig) {
    let is_image_mode = cfg.style == CrosshairStyle::Image;

    // 1. Sincronizza i radio button della modalità principale
    RADIO_MODE_STANDARD.with(|r| {
        SendMessageW(
            *r.borrow(),
            BM_SETCHECK,
            WPARAM(if is_image_mode { BST_UNCHECKED } else { BST_CHECKED }),
            LPARAM(0),
        );
    });
    RADIO_MODE_IMAGE.with(|r| {
        SendMessageW(
            *r.borrow(),
            BM_SETCHECK,
            WPARAM(if is_image_mode { BST_CHECKED } else { BST_UNCHECKED }),
            LPARAM(0),
        );
    });

    if is_image_mode {
        // =====================================================================
        // MODALITÀ IMMAGINE PERSONALIZZATA:
        // Nasconde tutti i controlli del mirino standard
        // =====================================================================
        show_ctrl(&LBL_COLOR_HEADER, false);
        BTN_COLORS.with(|btns| {
            for b in btns.borrow().iter() {
                let _ = ShowWindow(*b, SW_HIDE);
            }
        });
        show_ctrl(&LBL_COLOR_CURRENT, false);

        show_ctrl(&LBL_SHAPE_HEADER, false);
        show_ctrl(&RADIO_CROSS, false);
        show_ctrl(&RADIO_CIRCLE, false);
        show_ctrl(&RADIO_DOT, false);

        show_ctrl(&LBL_THICKNESS, false);
        show_ctrl(&SLIDER_THICKNESS, false);

        show_ctrl(&LBL_GAP, false);
        show_ctrl(&SLIDER_GAP, false);

        show_ctrl(&CHK_OUTLINE, false);
        show_ctrl(&CHK_DOT, false);

        show_ctrl(&LBL_DOT_SIZE, false);
        show_ctrl(&SLIDER_DOT_SIZE, false);

        // Mostra i controlli della libreria immagini
        show_ctrl(&LBL_CUSTOM_HEADER, true);
        show_ctrl(&BTN_IMPORT_H, true);
        show_ctrl(&BTN_OPEN_FOLDER_H, true);
        show_ctrl(&BTN_DELETE_H, true);
        show_ctrl(&LIST_CUSTOM, true);
        show_ctrl(&LBL_ACTIVE_IMAGE, true);

        // Riposiziona i cursori applicabili all'immagine (Dimensione & Opacità)
        move_ctrl(&LBL_SIZE, 20, 310, 430, 16);
        move_ctrl(&SLIDER_SIZE, 20, 328, 430, 26);
        LBL_SIZE.with(|lbl| {
            update_label(*lbl.borrow(), &format!("Dimensione / Scala Immagine (Size): {} px", cfg.size));
        });

        let op_pct = ((cfg.opacity as f32 / 255.0) * 100.0).round() as i32;
        move_ctrl(&LBL_OPACITY, 20, 360, 430, 16);
        move_ctrl(&SLIDER_OPACITY, 20, 378, 430, 26);
        LBL_OPACITY.with(|lbl| {
            update_label(*lbl.borrow(), &format!("Opacità Immagine (Opacity): {}%", op_pct));
        });

        move_ctrl(&LBL_TIP, 20, 414, 430, 28);
        LBL_TIP.with(|lbl| {
            update_label(*lbl.borrow(), "Trascina immagini (.png, .svg, .webp) nella cartella per usarle subito.");
        });

        // Adatta l'altezza della finestra (compatta e pulita)
        let _ = SetWindowPos(hwnd, None, 0, 0, 490, 490, SWP_NOMOVE | SWP_NOZORDER);
    } else {
        // =====================================================================
        // MODALITÀ MIRINO STANDARD:
        // Nasconde i controlli della libreria immagini
        // =====================================================================
        show_ctrl(&LBL_CUSTOM_HEADER, false);
        show_ctrl(&BTN_IMPORT_H, false);
        show_ctrl(&BTN_OPEN_FOLDER_H, false);
        show_ctrl(&BTN_DELETE_H, false);
        show_ctrl(&LIST_CUSTOM, false);
        show_ctrl(&LBL_ACTIVE_IMAGE, false);

        // Mostra i controlli del mirino standard
        // 1. Colore (in alto)
        show_ctrl(&LBL_COLOR_HEADER, true);
        BTN_COLORS.with(|btns| {
            for b in btns.borrow().iter() {
                let _ = ShowWindow(*b, SW_SHOW);
            }
        });
        show_ctrl(&LBL_COLOR_CURRENT, true);
        update_color_preview_label(cfg.color);

        // 2. Forme
        show_ctrl(&LBL_SHAPE_HEADER, true);
        show_ctrl(&RADIO_CROSS, true);
        show_ctrl(&RADIO_CIRCLE, true);
        show_ctrl(&RADIO_DOT, true);

        RADIO_CROSS.with(|r| {
            SendMessageW(
                *r.borrow(),
                BM_SETCHECK,
                WPARAM(if cfg.style == CrosshairStyle::Cross { BST_CHECKED } else { BST_UNCHECKED }),
                LPARAM(0),
            );
        });
        RADIO_CIRCLE.with(|r| {
            SendMessageW(
                *r.borrow(),
                BM_SETCHECK,
                WPARAM(if cfg.style == CrosshairStyle::Circle { BST_CHECKED } else { BST_UNCHECKED }),
                LPARAM(0),
            );
        });
        RADIO_DOT.with(|r| {
            SendMessageW(
                *r.borrow(),
                BM_SETCHECK,
                WPARAM(if cfg.style == CrosshairStyle::Dot { BST_CHECKED } else { BST_UNCHECKED }),
                LPARAM(0),
            );
        });

        // 3. Dimensione
        move_ctrl(&LBL_SIZE, 20, 242, 430, 16);
        move_ctrl(&SLIDER_SIZE, 20, 260, 430, 26);
        LBL_SIZE.with(|lbl| {
            update_label(*lbl.borrow(), &format!("Dimensione Bracci / Raggio (Size): {} px", cfg.size));
        });

        // 4. Spessore
        show_ctrl(&LBL_THICKNESS, true);
        show_ctrl(&SLIDER_THICKNESS, true);
        let is_dot = cfg.style == CrosshairStyle::Dot;
        let is_circle = cfg.style == CrosshairStyle::Circle;
        SLIDER_THICKNESS.with(|s| {
            let _ = EnableWindow(*s.borrow(), (!is_dot).into());
        });
        LBL_THICKNESS.with(|l| {
            let _ = EnableWindow(*l.borrow(), (!is_dot).into());
        });

        // 5. Spazio centrale (Gap)
        show_ctrl(&LBL_GAP, true);
        show_ctrl(&SLIDER_GAP, true);
        SLIDER_GAP.with(|s| {
            let _ = EnableWindow(*s.borrow(), (!is_circle && !is_dot).into());
        });
        LBL_GAP.with(|l| {
            let _ = EnableWindow(*l.borrow(), (!is_circle && !is_dot).into());
        });

        // 6. Checkbox Bordo nero & Punto centrale
        show_ctrl(&CHK_OUTLINE, true);
        show_ctrl(&CHK_DOT, true);
        CHK_DOT.with(|c| {
            let _ = EnableWindow(*c.borrow(), (!is_dot).into());
        });

        // 7. Dimensione punto centrale
        show_ctrl(&LBL_DOT_SIZE, true);
        show_ctrl(&SLIDER_DOT_SIZE, true);
        let dot_active = cfg.dot || is_dot;
        SLIDER_DOT_SIZE.with(|s| {
            let _ = EnableWindow(*s.borrow(), dot_active.into());
        });
        LBL_DOT_SIZE.with(|l| {
            let _ = EnableWindow(*l.borrow(), dot_active.into());
        });

        // 8. Opacità
        move_ctrl(&LBL_OPACITY, 20, 462, 430, 16);
        move_ctrl(&SLIDER_OPACITY, 20, 480, 430, 26);
        let op_pct = ((cfg.opacity as f32 / 255.0) * 100.0).round() as i32;
        LBL_OPACITY.with(|lbl| {
            update_label(*lbl.borrow(), &format!("Opacità (Opacity): {}%", op_pct));
        });

        // 9. Tip
        move_ctrl(&LBL_TIP, 20, 516, 430, 28);
        LBL_TIP.with(|lbl| {
            update_label(*lbl.borrow(), "Tasto destro sull'icona nella barra di sistema per il menu rapido e Spegnimento.");
        });

        // Adatta l'altezza della finestra (spaziosa e completa)
        let _ = SetWindowPos(hwnd, None, 0, 0, 490, 595, SWP_NOMOVE | SWP_NOZORDER);
    }

    let _ = InvalidateRect(hwnd, None, true);
}

/// Popola o aggiorna l'elenco dei monitor nel menu a tendina
pub unsafe fn refresh_monitors_list() {
    let monitors = enumerate_monitors();
    let cfg = crate::get_config();

    LBL_MONITOR_HEADER.with(|l| {
        let hwnd = *l.borrow();
        if !hwnd.0.is_null() {
            let title = if monitors.len() > 1 {
                format!("SCHERMO DI VISUALIZZAZIONE ({} monitor rilevati):", monitors.len())
            } else {
                "SCHERMO DI VISUALIZZAZIONE (1 monitor rilevato - Principale):".to_string()
            };
            update_label(hwnd, &title);
        }
    });

    COMBO_MONITOR.with(|cb| {
        let hwnd = *cb.borrow();
        if hwnd.0.is_null() {
            return;
        }

        let _ = SendMessageW(hwnd, CB_RESETCONTENT, WPARAM(0), LPARAM(0));

        for mon in &monitors {
            let wide = to_wide(&mon.name);
            let _ = SendMessageW(hwnd, CB_ADDSTRING, WPARAM(0), LPARAM(wide.as_ptr() as isize));
        }

        let sel = if cfg.monitor_index < monitors.len() {
            cfg.monitor_index
        } else {
            0
        };
        let _ = SendMessageW(hwnd, CB_SETCURSEL, WPARAM(sel), LPARAM(0));
    });
}

/// Aggiorna la ListBox con tutti i file di mirini disponibili nella cartella custom_crosshairs/
pub unsafe fn refresh_custom_list() {
    LIST_CUSTOM.with(|lb| {
        let hwnd = *lb.borrow();
        if hwnd.0.is_null() {
            return;
        }

        let _ = SendMessageW(hwnd, LB_RESETCONTENT, WPARAM(0), LPARAM(0));

        let folder = CrosshairConfig::custom_crosshairs_dir();
        let mut files = Vec::new();

        if let Ok(entries) = std::fs::read_dir(&folder) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_file() {
                    let ext = path
                        .extension()
                        .and_then(|e| e.to_str())
                        .unwrap_or("")
                        .to_lowercase();
                    if matches!(ext.as_str(), "svg" | "png" | "webp" | "jpg" | "jpeg" | "bmp") {
                        if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                            files.push(name.to_string());
                        }
                    }
                }
            }
        }

        files.sort_by(|a, b| {
            if a == "logo.svg" {
                std::cmp::Ordering::Less
            } else if b == "logo.svg" {
                std::cmp::Ordering::Greater
            } else {
                a.cmp(b)
            }
        });

        let cfg = crate::get_config();
        let current_file = cfg
            .image_path
            .as_ref()
            .and_then(|p| Path::new(p).file_name())
            .and_then(|n| n.to_str())
            .unwrap_or("");

        let mut sel_idx = 0;
        for (i, file) in files.iter().enumerate() {
            let wide = to_wide(file);
            let _ = SendMessageW(
                hwnd,
                LB_ADDSTRING,
                WPARAM(0),
                LPARAM(wide.as_ptr() as isize),
            );
            if file == current_file {
                sel_idx = i;
            }
        }

        if !files.is_empty() {
            let _ = SendMessageW(hwnd, LB_SETCURSEL, WPARAM(sel_idx), LPARAM(0));
        }
    });
}

unsafe fn create_label(
    parent: HWND,
    hinstance: HINSTANCE,
    text: &str,
    x: i32,
    y: i32,
    w: i32,
    h: i32,
    font: HGDIOBJ,
) -> HWND {
    let wide_text = to_wide(text);
    let cls = to_wide("STATIC");
    let hwnd = CreateWindowExW(
        WINDOW_EX_STYLE::default(),
        PCWSTR(cls.as_ptr()),
        PCWSTR(wide_text.as_ptr()),
        WS_CHILD | WS_VISIBLE,
        x,
        y,
        w,
        h,
        parent,
        None,
        hinstance,
        None,
    )
    .unwrap();
    let _ = SendMessageW(hwnd, WM_SETFONT, WPARAM(font.0 as usize), LPARAM(1));
    hwnd
}

unsafe fn create_button(
    parent: HWND,
    hinstance: HINSTANCE,
    text: &str,
    x: i32,
    y: i32,
    w: i32,
    h: i32,
    id: u32,
    font: HGDIOBJ,
) -> HWND {
    let wide_text = to_wide(text);
    let cls = to_wide("BUTTON");
    let hwnd = CreateWindowExW(
        WINDOW_EX_STYLE::default(),
        PCWSTR(cls.as_ptr()),
        PCWSTR(wide_text.as_ptr()),
        WS_CHILD | WS_VISIBLE | WS_TABSTOP | WINDOW_STYLE(BS_PUSHBUTTON as u32),
        x,
        y,
        w,
        h,
        parent,
        HMENU(id as *mut std::ffi::c_void),
        hinstance,
        None,
    )
    .unwrap();
    let _ = SendMessageW(hwnd, WM_SETFONT, WPARAM(font.0 as usize), LPARAM(1));
    hwnd
}

unsafe fn create_radio(
    parent: HWND,
    hinstance: HINSTANCE,
    text: &str,
    x: i32,
    y: i32,
    w: i32,
    h: i32,
    id: u32,
    font: HGDIOBJ,
) -> HWND {
    let wide_text = to_wide(text);
    let cls = to_wide("BUTTON");
    let hwnd = CreateWindowExW(
        WINDOW_EX_STYLE::default(),
        PCWSTR(cls.as_ptr()),
        PCWSTR(wide_text.as_ptr()),
        WS_CHILD | WS_VISIBLE | WINDOW_STYLE(BS_AUTORADIOBUTTON as u32),
        x,
        y,
        w,
        h,
        parent,
        HMENU(id as *mut std::ffi::c_void),
        hinstance,
        None,
    )
    .unwrap();
    let _ = SendMessageW(hwnd, WM_SETFONT, WPARAM(font.0 as usize), LPARAM(1));
    hwnd
}

unsafe fn create_checkbox(
    parent: HWND,
    hinstance: HINSTANCE,
    text: &str,
    x: i32,
    y: i32,
    w: i32,
    h: i32,
    id: u32,
    font: HGDIOBJ,
) -> HWND {
    let wide_text = to_wide(text);
    let cls = to_wide("BUTTON");
    let hwnd = CreateWindowExW(
        WINDOW_EX_STYLE::default(),
        PCWSTR(cls.as_ptr()),
        PCWSTR(wide_text.as_ptr()),
        WS_CHILD | WS_VISIBLE | WINDOW_STYLE(BS_AUTOCHECKBOX as u32),
        x,
        y,
        w,
        h,
        parent,
        HMENU(id as *mut std::ffi::c_void),
        hinstance,
        None,
    )
    .unwrap();
    let _ = SendMessageW(hwnd, WM_SETFONT, WPARAM(font.0 as usize), LPARAM(1));
    hwnd
}

unsafe fn create_slider(
    parent: HWND,
    hinstance: HINSTANCE,
    x: i32,
    y: i32,
    w: i32,
    h: i32,
    id: u32,
    min: i32,
    max: i32,
    val: i32,
) -> HWND {
    let hwnd = CreateWindowExW(
        WINDOW_EX_STYLE::default(),
        TRACKBAR_CLASSW,
        PCWSTR::null(),
        WS_CHILD | WS_VISIBLE | WS_TABSTOP | WINDOW_STYLE(TBS_HORZ as u32 | TBS_AUTOTICKS as u32),
        x,
        y,
        w,
        h,
        parent,
        HMENU(id as *mut std::ffi::c_void),
        hinstance,
        None,
    )
    .unwrap();

    let range = ((max as u32) << 16) | (min as u32 & 0xFFFF);
    SendMessageW(hwnd, TBM_SETRANGE, WPARAM(1), LPARAM(range as isize));
    SendMessageW(hwnd, TBM_SETPOS, WPARAM(1), LPARAM(val as isize));
    hwnd
}

unsafe fn update_label(lbl: HWND, text: &str) {
    if !lbl.0.is_null() {
        let wide = to_wide(text);
        let _ = SetWindowTextW(lbl, PCWSTR(wide.as_ptr()));
    }
}

pub fn open_image_dialog(parent: HWND) -> Option<String> {
    unsafe {
        let mut file_buf: [u16; 512] = [0; 512];
        let filter = to_wide("Immagini supportate (*.png;*.webp;*.svg;*.jpg;*.jpeg;*.bmp)\0*.png;*.webp;*.svg;*.jpg;*.jpeg;*.bmp\0Tutti i file (*.*)\0*.*\0\0");
        let title = to_wide("Seleziona Immagine per il Mirino");

        let mut ofn: OPENFILENAMEW = mem::zeroed();
        ofn.lStructSize = mem::size_of::<OPENFILENAMEW>() as u32;
        ofn.hwndOwner = parent;
        ofn.lpstrFilter = PCWSTR(filter.as_ptr());
        ofn.lpstrFile = PWSTR(file_buf.as_mut_ptr());
        ofn.nMaxFile = file_buf.len() as u32;
        ofn.lpstrTitle = PCWSTR(title.as_ptr());
        ofn.Flags = OFN_FILEMUSTEXIST | OFN_PATHMUSTEXIST;

        if GetOpenFileNameW(&mut ofn).as_bool() {
            let len = file_buf.iter().position(|&c| c == 0).unwrap_or(file_buf.len());
            let path_str = String::from_utf16_lossy(&file_buf[..len]);
            if !path_str.trim().is_empty() {
                return Some(path_str);
            }
        }
    }
    None
}

unsafe fn pick_color_dialog(parent: HWND, current: Color) -> Option<Color> {
    let mut custom_colors = [COLORREF(0); 16];
    let mut cc: CHOOSECOLORW = mem::zeroed();
    cc.lStructSize = mem::size_of::<CHOOSECOLORW>() as u32;
    cc.hwndOwner = parent;
    cc.rgbResult = COLORREF(current.to_colorref());
    cc.lpCustColors = custom_colors.as_mut_ptr();
    cc.Flags = CC_RGBINIT | CC_FULLOPEN;

    if ChooseColorW(&mut cc).as_bool() {
        let cr = cc.rgbResult.0;
        let r = (cr & 0xFF) as u8;
        let g = ((cr >> 8) & 0xFF) as u8;
        let b = ((cr >> 16) & 0xFF) as u8;
        Some(Color { r, g, b })
    } else {
        None
    }
}

pub unsafe extern "system" fn settings_wnd_proc(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    match msg {
        WM_CREATE => {
            let hinstance = GetModuleHandleW(None).unwrap().into();
            create_controls(hwnd, hinstance);
            LRESULT(0)
        }

        WM_APP_UPDATE => {
            let cfg = crate::get_config();
            refresh_monitors_list();
            refresh_custom_list();
            update_ui_layout_and_visibility(hwnd, &cfg);
            LRESULT(0)
        }

        WM_HSCROLL => {
            let slider_hwnd = HWND(lparam.0 as *mut std::ffi::c_void);
            let val = SendMessageW(slider_hwnd, TBM_GETPOS, WPARAM(0), LPARAM(0)).0 as i32;

            let mut cfg = crate::get_config();

            SLIDER_SIZE.with(|s| {
                if slider_hwnd == *s.borrow() {
                    cfg.size = val as u32;
                    let is_img = cfg.style == CrosshairStyle::Image;
                    let label_text = if is_img {
                        format!("Dimensione / Scala Immagine (Size): {} px", val)
                    } else {
                        format!("Dimensione Bracci / Raggio (Size): {} px", val)
                    };
                    LBL_SIZE.with(|lbl| update_label(*lbl.borrow(), &label_text));
                }
            });

            SLIDER_THICKNESS.with(|s| {
                if slider_hwnd == *s.borrow() {
                    cfg.thickness = val as u32;
                    LBL_THICKNESS.with(|lbl| update_label(*lbl.borrow(), &format!("Spessore linee (Thickness): {} px", val)));
                }
            });

            SLIDER_GAP.with(|s| {
                if slider_hwnd == *s.borrow() {
                    cfg.gap = val as u32;
                    LBL_GAP.with(|lbl| update_label(*lbl.borrow(), &format!("Spazio centrale (Center Gap): {} px", val)));
                }
            });

            SLIDER_DOT_SIZE.with(|s| {
                if slider_hwnd == *s.borrow() {
                    cfg.dot_size = val as u32;
                    LBL_DOT_SIZE.with(|lbl| update_label(*lbl.borrow(), &format!("Dimensione punto centrale: {} px", val)));
                }
            });

            SLIDER_OPACITY.with(|s| {
                if slider_hwnd == *s.borrow() {
                    let alpha = ((val as f32 / 100.0) * 255.0).round() as u8;
                    cfg.opacity = alpha;
                    let is_img = cfg.style == CrosshairStyle::Image;
                    let label_text = if is_img {
                        format!("Opacità Immagine (Opacity): {}%", val)
                    } else {
                        format!("Opacità (Opacity): {}%", val)
                    };
                    LBL_OPACITY.with(|lbl| update_label(*lbl.borrow(), &label_text));
                }
            });

            crate::set_config(cfg);
            notify_overlay_update();
            LRESULT(0)
        }

        WM_COMMAND => {
            let id = (wparam.0 & 0xFFFF) as u32;
            let code = ((wparam.0 >> 16) & 0xFFFF) as u16;
            let mut cfg = crate::get_config();
            let mut changed = true;

            match id {
                ID_COMBO_MONITOR => {
                    if code == CBN_SELCHANGE {
                        COMBO_MONITOR.with(|cb| {
                            let cur_sel = SendMessageW(*cb.borrow(), CB_GETCURSEL, WPARAM(0), LPARAM(0)).0 as i32;
                            if cur_sel >= 0 {
                                cfg.monitor_index = cur_sel as usize;
                            }
                        });
                    } else {
                        changed = false;
                    }
                }

                // --- MODALITÀ: Standard vs Custom Image ---
                ID_MODE_STANDARD => {
                    if cfg.style == CrosshairStyle::Image {
                        cfg.style = CrosshairStyle::Cross;
                    }
                    update_ui_layout_and_visibility(hwnd, &cfg);
                }
                ID_MODE_IMAGE => {
                    cfg.style = CrosshairStyle::Image;
                    if cfg.image_path.is_none() {
                        let default_logo = CrosshairConfig::custom_crosshairs_dir().join("logo.svg");
                        cfg.image_path = Some(default_logo.to_string_lossy().to_string());
                    }
                    update_ui_layout_and_visibility(hwnd, &cfg);
                }

                // --- FORME STANDARD ---
                ID_RADIO_CROSS => {
                    cfg.style = CrosshairStyle::Cross;
                    update_ui_layout_and_visibility(hwnd, &cfg);
                }
                ID_RADIO_CIRCLE => {
                    cfg.style = CrosshairStyle::Circle;
                    update_ui_layout_and_visibility(hwnd, &cfg);
                }
                ID_RADIO_DOT => {
                    cfg.style = CrosshairStyle::Dot;
                    update_ui_layout_and_visibility(hwnd, &cfg);
                }

                // --- CHECKBOXES ---
                ID_CHECK_OUTLINE => {
                    CHK_OUTLINE.with(|chk| {
                        let state = SendMessageW(*chk.borrow(), BM_GETCHECK, WPARAM(0), LPARAM(0)).0 as usize;
                        cfg.outline = state == BST_CHECKED;
                    });
                }
                ID_CHECK_DOT => {
                    CHK_DOT.with(|chk| {
                        let state = SendMessageW(*chk.borrow(), BM_GETCHECK, WPARAM(0), LPARAM(0)).0 as usize;
                        cfg.dot = state == BST_CHECKED;
                    });
                    update_ui_layout_and_visibility(hwnd, &cfg);
                }

                // --- COLORI STANDARD (PORTATI IN ALTO) ---
                ID_BTN_COLOR_GREEN => {
                    cfg.color = Color::GREEN;
                    update_color_preview_label(cfg.color);
                }
                ID_BTN_COLOR_RED => {
                    cfg.color = Color::RED;
                    update_color_preview_label(cfg.color);
                }
                ID_BTN_COLOR_CYAN => {
                    cfg.color = Color::CYAN;
                    update_color_preview_label(cfg.color);
                }
                ID_BTN_COLOR_YELLOW => {
                    cfg.color = Color::YELLOW;
                    update_color_preview_label(cfg.color);
                }
                ID_BTN_COLOR_WHITE => {
                    cfg.color = Color::WHITE;
                    update_color_preview_label(cfg.color);
                }
                ID_BTN_COLOR_PICKER => {
                    if let Some(c) = pick_color_dialog(hwnd, cfg.color) {
                        cfg.color = c;
                        update_color_preview_label(cfg.color);
                    } else {
                        changed = false;
                    }
                }

                // --- LIBRERIA IMMAGINI PERSONALIZZATE ---
                ID_BTN_IMPORT => {
                    if let Some(src_path_str) = open_image_dialog(hwnd) {
                        let src_path = Path::new(&src_path_str);
                        if let Some(file_name) = src_path.file_name() {
                            let target_folder = CrosshairConfig::custom_crosshairs_dir();
                            let target_path = target_folder.join(file_name);
                            let _ = std::fs::copy(src_path, &target_path);

                            cfg.image_path = Some(target_path.to_string_lossy().to_string());
                            cfg.style = CrosshairStyle::Image;

                            refresh_custom_list();
                            LBL_ACTIVE_IMAGE.with(|lbl| {
                                update_label(*lbl.borrow(), &format!("File attivo: {}", file_name.to_string_lossy()));
                            });
                            update_ui_layout_and_visibility(hwnd, &cfg);
                        }
                    } else {
                        changed = false;
                    }
                }
                ID_BTN_OPEN_FOLDER => {
                    let folder = CrosshairConfig::custom_crosshairs_dir();
                    let folder_str = to_wide(&folder.to_string_lossy());
                    let op = to_wide("open");
                    let _ = ShellExecuteW(
                        None,
                        PCWSTR(op.as_ptr()),
                        PCWSTR(folder_str.as_ptr()),
                        PCWSTR::null(),
                        PCWSTR::null(),
                        SW_SHOWNORMAL,
                    );
                    changed = false;
                }
                ID_BTN_DELETE => {
                    LIST_CUSTOM.with(|lb| {
                        let h_lb = *lb.borrow();
                        let cur_sel = SendMessageW(h_lb, LB_GETCURSEL, WPARAM(0), LPARAM(0)).0 as i32;
                        if cur_sel >= 0 {
                            let mut buf: [u16; 260] = [0; 260];
                            let len = SendMessageW(
                                h_lb,
                                LB_GETTEXT,
                                WPARAM(cur_sel as usize),
                                LPARAM(buf.as_mut_ptr() as isize),
                            )
                            .0 as usize;
                            let fname = String::from_utf16_lossy(&buf[..len]);

                            let confirm_msg = format!(
                                "Sei sicuro di voler eliminare '{}'?\nL'azione non può essere annullata.",
                                fname
                            );
                            let title = to_wide("Matt's Crosshair — Conferma Eliminazione");
                            let msg_wide = to_wide(&confirm_msg);

                            let res = MessageBoxW(
                                hwnd,
                                PCWSTR(msg_wide.as_ptr()),
                                PCWSTR(title.as_ptr()),
                                MB_YESNO | MB_ICONWARNING | MB_DEFBUTTON2,
                            );
                            if res == IDYES {
                                let folder = CrosshairConfig::custom_crosshairs_dir();
                                let file_path = folder.join(&fname);
                                let _ = std::fs::remove_file(file_path);
                                refresh_custom_list();

                                let fallback = folder.join("logo.svg");
                                cfg.image_path = Some(fallback.to_string_lossy().to_string());
                                LBL_ACTIVE_IMAGE.with(|lbl| update_label(*lbl.borrow(), "File attivo: logo.svg"));
                            }
                        }
                    });
                }
                ID_LIST_CUSTOM => {
                    if code == LBN_SELCHANGE {
                        LIST_CUSTOM.with(|lb| {
                            let h_lb = *lb.borrow();
                            let cur_sel = SendMessageW(h_lb, LB_GETCURSEL, WPARAM(0), LPARAM(0)).0 as i32;
                            if cur_sel >= 0 {
                                let mut buf: [u16; 260] = [0; 260];
                                let len = SendMessageW(
                                    h_lb,
                                    LB_GETTEXT,
                                    WPARAM(cur_sel as usize),
                                    LPARAM(buf.as_mut_ptr() as isize),
                                )
                                .0 as usize;
                                let fname = String::from_utf16_lossy(&buf[..len]);
                                let folder = CrosshairConfig::custom_crosshairs_dir();
                                let target_path = folder.join(&fname);

                                cfg.image_path = Some(target_path.to_string_lossy().to_string());
                                cfg.style = CrosshairStyle::Image;

                                LBL_ACTIVE_IMAGE.with(|lbl| {
                                    update_label(*lbl.borrow(), &format!("File attivo: {}", fname));
                                });
                                update_ui_layout_and_visibility(hwnd, &cfg);
                            }
                        });
                    } else {
                        changed = false;
                    }
                }
                _ => {
                    changed = false;
                }
            }

            if changed {
                crate::set_config(cfg);
                notify_overlay_update();
            }

            LRESULT(0)
        }

        WM_CLOSE => {
            let _ = ShowWindow(hwnd, SW_HIDE);
            LRESULT(0)
        }

        _ => DefWindowProcW(hwnd, msg, wparam, lparam),
    }
}
