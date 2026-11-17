use std::path::Path;

/// Loads an image (PNG, WEBP, SVG, JPG, BMP) and returns (width, height, premultiplied_bgra_bytes)
pub fn load_image_bgra(path: &str, target_size: u32) -> Result<(u32, u32, Vec<u8>), String> {
    let p = Path::new(path);
    if !p.exists() {
        return Err(format!("File not found: {}", path));
    }

    let ext = p.extension().and_then(|e| e.to_str()).unwrap_or("").to_lowercase();

    if ext == "svg" {
        load_svg(p, target_size)
    } else {
        load_raster(p, target_size)
    }
}

fn load_svg(path: &Path, target_size: u32) -> Result<(u32, u32, Vec<u8>), String> {
    let data = std::fs::read(path).map_err(|e| format!("Failed to read SVG: {}", e))?;
    let opt = resvg::usvg::Options::default();
    let tree = resvg::usvg::Tree::from_data(&data, &opt)
        .map_err(|e| format!("Invalid SVG: {}", e))?;

    let size = tree.size();
    let (sw, sh) = (size.width(), size.height());
    let target = target_size.clamp(4, 300) as f32;
    let scale = target / sw.max(sh).max(1.0);

    let out_w = ((sw * scale).round() as u32).max(1);
    let out_h = ((sh * scale).round() as u32).max(1);

    let mut pixmap = resvg::tiny_skia::Pixmap::new(out_w, out_h)
        .ok_or_else(|| "Failed to allocate pixmap".to_string())?;

    let transform = resvg::tiny_skia::Transform::from_scale(scale, scale);
    resvg::render(&tree, transform, &mut pixmap.as_mut());

    // Convert RGBA premultiplied -> BGRA premultiplied
    let src = pixmap.data();
    let mut bgra = Vec::with_capacity(src.len());
    for chunk in src.chunks_exact(4) {
        let (r, g, b, a) = (chunk[0], chunk[1], chunk[2], chunk[3]);
        bgra.push(b);
        bgra.push(g);
        bgra.push(r);
        bgra.push(a);
    }

    Ok((out_w, out_h, bgra))
}

fn load_raster(path: &Path, target_size: u32) -> Result<(u32, u32, Vec<u8>), String> {
    // image::open supports PNG, WEBP, JPEG, BMP
    let img = image::open(path).map_err(|e| format!("Failed to open image: {}", e))?;
    let (orig_w, orig_h) = (img.width(), img.height());

    let target = target_size.clamp(4, 300) as f32;
    let scale = target / (orig_w.max(orig_h) as f32).max(1.0);

    let out_w = ((orig_w as f32 * scale).round() as u32).max(1);
    let out_h = ((orig_h as f32 * scale).round() as u32).max(1);

    let resized = img.resize(out_w, out_h, image::imageops::FilterType::Lanczos3);
    let rgba = resized.to_rgba8();

    let mut bgra = Vec::with_capacity((out_w * out_h * 4) as usize);
    for pixel in rgba.pixels() {
        let (r, g, b, a) = (pixel[0], pixel[1], pixel[2], pixel[3]);
        // Premultiply RGB with Alpha for Windows 32-bit DIB
        let a_f = a as f32 / 255.0;
        let pr = (r as f32 * a_f).round() as u8;
        let pg = (g as f32 * a_f).round() as u8;
        let pb = (b as f32 * a_f).round() as u8;
        bgra.push(pb);
        bgra.push(pg);
        bgra.push(pr);
        bgra.push(a);
    }

    Ok((out_w, out_h, bgra))
}
