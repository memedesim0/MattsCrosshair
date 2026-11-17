use std::mem;
use windows::{
    Win32::Foundation::*,
    Win32::Graphics::Gdi::*,
    Win32::UI::WindowsAndMessaging::*,
};

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct MonitorInfo {
    pub index: usize,
    pub rect: RECT,
    pub is_primary: bool,
    pub name: String,
    pub width: i32,
    pub height: i32,
}

pub unsafe fn enumerate_monitors() -> Vec<MonitorInfo> {
    let mut monitors: Vec<MonitorInfo> = Vec::new();

    unsafe extern "system" fn enum_proc(
        hmon: HMONITOR,
        _hdc: HDC,
        _rect: *mut RECT,
        lparam: LPARAM,
    ) -> BOOL {
        let list = &mut *(lparam.0 as *mut Vec<MonitorInfo>);
        let mut mi: MONITORINFOEXW = mem::zeroed();
        mi.monitorInfo.cbSize = mem::size_of::<MONITORINFOEXW>() as u32;

        if GetMonitorInfoW(hmon, &mut mi.monitorInfo as *mut _ as *mut MONITORINFO).as_bool() {
            let rc = mi.monitorInfo.rcMonitor;
            let w = rc.right - rc.left;
            let h = rc.bottom - rc.top;
            let is_primary = (mi.monitorInfo.dwFlags & MONITORINFOF_PRIMARY) != 0;

            let idx = list.len();
            let label = if is_primary {
                format!("Monitor {} ({}x{}) [Primary]", idx + 1, w, h)
            } else {
                format!("Monitor {} ({}x{})", idx + 1, w, h)
            };

            list.push(MonitorInfo {
                index: idx,
                rect: rc,
                is_primary,
                name: label,
                width: w,
                height: h,
            });
        }
        BOOL(1)
    }

    let _ = EnumDisplayMonitors(
        None,
        None,
        Some(enum_proc),
        LPARAM(&mut monitors as *mut _ as isize),
    );

    if monitors.is_empty() {
        let sw = GetSystemMetrics(SM_CXSCREEN);
        let sh = GetSystemMetrics(SM_CYSCREEN);
        monitors.push(MonitorInfo {
            index: 0,
            rect: RECT { left: 0, top: 0, right: sw, bottom: sh },
            is_primary: true,
            name: format!("Monitor 1 ({}x{}) [Primary]", sw, sh),
            width: sw,
            height: sh,
        });
    }

    monitors
}

/// Returns the exact center point (x, y) of the requested monitor in virtual screen coordinates
pub unsafe fn get_monitor_center(monitor_index: usize) -> (i32, i32) {
    let monitors = enumerate_monitors();
    let mon = monitors.get(monitor_index).or_else(|| monitors.first()).unwrap();
    let cx = mon.rect.left + (mon.width / 2);
    let cy = mon.rect.top + (mon.height / 2);
    (cx, cy)
}
