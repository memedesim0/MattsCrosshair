use crate::config::{Color, CrosshairConfig, CrosshairStyle};
use crate::image_loader::load_image_bgra;

pub fn render_crosshair(buffer: &mut [u8], width: u32, height: u32, cfg: &CrosshairConfig) {
    // Clear buffer to completely transparent (0)
    buffer.fill(0);

    let cx = (width / 2) as i32;
    let cy = (height / 2) as i32;
    let w = width as i32;
    let h = height as i32;

    match cfg.style {
        CrosshairStyle::Cross => {
            draw_cross(buffer, w, h, cx, cy, cfg);
            if cfg.dot {
                draw_dot(buffer, w, h, cx, cy, cfg.dot_size.max(1) as i32, cfg.color, cfg.outline, cfg.outline_color);
            }
        }
        CrosshairStyle::Circle => {
            draw_circle(buffer, w, h, cx, cy, cfg);
            if cfg.dot {
                draw_dot(buffer, w, h, cx, cy, cfg.dot_size.max(1) as i32, cfg.color, cfg.outline, cfg.outline_color);
            }
        }
        CrosshairStyle::Dot => {
            // Precise competitive dot
            let dot_r = cfg.size.max(1) as i32;
            draw_dot(buffer, w, h, cx, cy, dot_r, cfg.color, cfg.outline, cfg.outline_color);
        }
        CrosshairStyle::Image => {
            let mut drawn = false;
            if let Some(ref path) = cfg.image_path {
                let img_size = cfg.size.clamp(4, 300);
                if let Ok((img_w, img_h, bgra)) = load_image_bgra(path, img_size) {
                    blit_image(buffer, w, h, cx, cy, img_w as i32, img_h as i32, &bgra);
                    drawn = true;
                }
            }
            if !drawn {
                draw_cross(buffer, w, h, cx, cy, cfg);
            }
            // Allow center dot to be added and customized on top of any custom image!
            if cfg.dot {
                draw_dot(buffer, w, h, cx, cy, cfg.dot_size.max(1) as i32, cfg.color, cfg.outline, cfg.outline_color);
            }
        }
    }
}

#[inline(always)]
fn set_pixel_alpha(buf: &mut [u8], w: i32, h: i32, x: i32, y: i32, color: Color, alpha: u8) {
    if x < 0 || x >= w || y < 0 || y >= h || alpha == 0 {
        return;
    }
    let idx = ((y * w + x) * 4) as usize;
    if idx + 3 >= buf.len() {
        return;
    }

    let a_f = alpha as f32 / 255.0;
    let pr = color.r as f32 * a_f;
    let pg = color.g as f32 * a_f;
    let pb = color.b as f32 * a_f;

    let existing_a = buf[idx + 3] as f32 / 255.0;
    let inv_a = 1.0 - a_f;

    buf[idx + 0] = (pb + buf[idx + 0] as f32 * inv_a).min(255.0) as u8;
    buf[idx + 1] = (pg + buf[idx + 1] as f32 * inv_a).min(255.0) as u8;
    buf[idx + 2] = (pr + buf[idx + 2] as f32 * inv_a).min(255.0) as u8;
    buf[idx + 3] = ((a_f + existing_a * inv_a) * 255.0).min(255.0) as u8;
}

fn fill_rect(buf: &mut [u8], w: i32, h: i32, x0: i32, y0: i32, x1: i32, y1: i32, color: Color) {
    let (min_x, max_x) = if x0 < x1 { (x0, x1) } else { (x1, x0) };
    let (min_y, max_y) = if y0 < y1 { (y0, y1) } else { (y1, y0) };

    for y in min_y..=max_y {
        for x in min_x..=max_x {
            set_pixel_alpha(buf, w, h, x, y, color, 255);
        }
    }
}

/// Precise dot (1px micro-dot, 2-3px sharp dot, or antialiased circle)
fn draw_dot(
    buf: &mut [u8],
    w: i32,
    h: i32,
    cx: i32,
    cy: i32,
    radius: i32,
    color: Color,
    outline: bool,
    outline_color: Color,
) {
    if radius == 1 {
        if outline {
            fill_rect(buf, w, h, cx - 1, cy - 1, cx + 1, cy + 1, outline_color);
        }
        set_pixel_alpha(buf, w, h, cx, cy, color, 255);
    } else if radius == 2 {
        if outline {
            fill_rect(buf, w, h, cx - 2, cy - 2, cx + 2, cy + 2, outline_color);
        }
        fill_rect(buf, w, h, cx - 1, cy - 1, cx + 1, cy + 1, color);
    } else {
        let r = radius as f32;
        let r_out = r + 1.2;
        let max_range = (r_out + 1.0).ceil() as i32;

        for dy in -max_range..=max_range {
            for dx in -max_range..=max_range {
                let dist = ((dx * dx + dy * dy) as f32).sqrt();

                if outline && dist > r - 0.5 && dist <= r_out {
                    let edge_alpha = if dist > r_out - 0.5 {
                        ((r_out - dist + 0.5) * 255.0).clamp(0.0, 255.0) as u8
                    } else {
                        255
                    };
                    set_pixel_alpha(buf, w, h, cx + dx, cy + dy, outline_color, edge_alpha);
                }

                if dist <= r {
                    let alpha = if dist > r - 0.7 {
                        ((r - dist + 0.7) * 255.0).clamp(0.0, 255.0) as u8
                    } else {
                        255
                    };
                    set_pixel_alpha(buf, w, h, cx + dx, cy + dy, color, alpha);
                }
            }
        }
    }
}

fn draw_cross(buf: &mut [u8], w: i32, h: i32, cx: i32, cy: i32, cfg: &CrosshairConfig) {
    let size = cfg.size as i32;
    let thickness = cfg.thickness.max(1) as i32;
    let gap = cfg.gap as i32;
    let half_t0 = thickness / 2;
    let half_t1 = thickness - half_t0 - 1;

    // Draw outline first if enabled
    if cfg.outline {
        let out_t0 = half_t0 + 1;
        let out_t1 = half_t1 + 1;

        // Left arm outline
        fill_rect(buf, w, h, cx - gap - size - 1, cy - out_t0, cx - gap + 1, cy + out_t1, cfg.outline_color);
        // Right arm outline
        fill_rect(buf, w, h, cx + gap - 1, cy - out_t0, cx + gap + size + 1, cy + out_t1, cfg.outline_color);
        // Top arm outline
        fill_rect(buf, w, h, cx - out_t0, cy - gap - size - 1, cx + out_t1, cy - gap + 1, cfg.outline_color);
        // Bottom arm outline
        fill_rect(buf, w, h, cx - out_t0, cy + gap - 1, cx + out_t1, cy + gap + size + 1, cfg.outline_color);
    }

    // Main arms
    fill_rect(buf, w, h, cx - gap - size, cy - half_t0, cx - gap, cy + half_t1, cfg.color);
    fill_rect(buf, w, h, cx + gap, cy - half_t0, cx + gap + size, cy + half_t1, cfg.color);
    fill_rect(buf, w, h, cx - half_t0, cy - gap - size, cx + half_t1, cy - gap, cfg.color);
    fill_rect(buf, w, h, cx - half_t0, cy + gap, cx + half_t1, cy + gap + size, cfg.color);
}

fn draw_circle(buf: &mut [u8], w: i32, h: i32, cx: i32, cy: i32, cfg: &CrosshairConfig) {
    let radius = cfg.size as f32;
    let thickness = cfg.thickness.max(1) as f32;
    let r_inner = (radius - thickness / 2.0).max(1.0);
    let r_outer = radius + thickness / 2.0;

    let max_range = (r_outer + 2.0).ceil() as i32;

    for dy in -max_range..=max_range {
        for dx in -max_range..=max_range {
            let dist = ((dx * dx + dy * dy) as f32).sqrt();

            if cfg.outline {
                let out_inner = (r_inner - 1.2).max(0.0);
                let out_outer = r_outer + 1.2;
                if (dist >= out_inner && dist <= r_inner) || (dist >= r_outer && dist <= out_outer) {
                    set_pixel_alpha(buf, w, h, cx + dx, cy + dy, cfg.outline_color, 255);
                }
            }

            if dist >= r_inner && dist <= r_outer {
                let mut alpha = 255f32;
                if dist < r_inner + 0.6 {
                    alpha = (dist - r_inner) / 0.6 * 255.0;
                } else if dist > r_outer - 0.6 {
                    alpha = (r_outer - dist) / 0.6 * 255.0;
                }
                set_pixel_alpha(buf, w, h, cx + dx, cy + dy, cfg.color, alpha.clamp(0.0, 255.0) as u8);
            }
        }
    }
}

fn blit_image(
    buf: &mut [u8],
    w: i32,
    h: i32,
    cx: i32,
    cy: i32,
    img_w: i32,
    img_h: i32,
    bgra: &[u8],
) {
    let start_x = cx - img_w / 2;
    let start_y = cy - img_h / 2;

    for iy in 0..img_h {
        let dy = start_y + iy;
        if dy < 0 || dy >= h {
            continue;
        }
        for ix in 0..img_w {
            let dx = start_x + ix;
            if dx < 0 || dx >= w {
                continue;
            }

            let src_idx = ((iy * img_w + ix) * 4) as usize;
            if src_idx + 3 >= bgra.len() {
                continue;
            }
            let pb = bgra[src_idx + 0];
            let pg = bgra[src_idx + 1];
            let pr = bgra[src_idx + 2];
            let a = bgra[src_idx + 3];

            if a == 0 {
                continue;
            }

            let dst_idx = ((dy * w + dx) * 4) as usize;
            if dst_idx + 3 >= buf.len() {
                continue;
            }

            let a_f = a as f32 / 255.0;
            let inv_a = 1.0 - a_f;

            buf[dst_idx + 0] = (pb as f32 + buf[dst_idx + 0] as f32 * inv_a).min(255.0) as u8;
            buf[dst_idx + 1] = (pg as f32 + buf[dst_idx + 1] as f32 * inv_a).min(255.0) as u8;
            buf[dst_idx + 2] = (pr as f32 + buf[dst_idx + 2] as f32 * inv_a).min(255.0) as u8;
            buf[dst_idx + 3] = ((a_f + (buf[dst_idx + 3] as f32 / 255.0) * inv_a) * 255.0).min(255.0) as u8;
        }
    }
}
