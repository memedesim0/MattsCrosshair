# Matt's Crosshair 🎯

An **ultra-lightweight, high-performance customizable crosshair overlay for Windows** created by **Matteo**.
- **Memory Footprint**: ~1.5–3 MB RAM
- **CPU Usage**: **0.0%** at idle (purely event-driven Win32 message loop, no polling)
- **Overlay Engine**: `UpdateLayeredWindow` with 32-bit premultiplied hardware-accelerated alpha blending

---

## ✨ Features

### 1. 🖼️ Custom Crosshairs Library (PNG, WEBP, SVG, BMP, JPG)
- **Dedicated Folder**: A `custom_crosshairs/` library folder is automatically managed.
- **Pre-installed Logo**: Comes with `logo.svg` and `logo.png` ready to use.
- **Instant Import**: Click **`➕ Import File...`** to pick any image: it is **automatically copied** into your crosshairs folder, so you can safely delete the original file from your desktop or downloads.
- **Drag & Drop Folder**: Click **`📂 Open Folder`** to open Windows File Explorer directly into the crosshairs directory and drop as many files as you like!
- **List & Delete**: Switch between all your saved crosshairs instantly from the list. Delete unwanted ones anytime with a confirmation dialog (**`🗑️ Delete`**).

### 2. 🎯 Center Dot on Custom Images
- You can now add and customize a **Center Dot** right on top of any custom image crosshair!
- Control its size, color, and outline independently.

### 3. 🧠 Smart Dynamic UI
- Controls that don't apply to the active style are automatically greyed out/disabled (e.g. thickness and gap are disabled when using an image, while scale/size and center dot remain active).

### 4. 🎚️ Live Sliders up to 200px
- **Size Slider**: 1 px up to **200 px**! Dragging the slider scales the crosshair on your screen **in real time**.
- **Thickness Slider**: 1 to 15 px.
- **Center Gap Slider**: 0 to 40 px.
- **Dot Size Slider**: 1 to 15 px.
- **Opacity Slider**: 20% to 100%.

### 5. 🎨 App Logo: Thick Red with White Outline
- Matteo's hand-drawn crosshair has been centered, transparent-backed, thickened, and outlined in crisp white so it pops on both dark and light Windows themes.
- Embedded as the native **System Tray Icon** and window icon.

### 6. ⏻ Remote "Shutdown" in Tray
- Quick shutdown option with the classic remote power-off symbol (**`⏻ Shutdown`**) in the tray menu.

---

## 🚀 How to Run

Run the release binary in PowerShell:

```powershell
.\target\release\matts_crosshair.exe
```

1. Look at your Windows taskbar **System Tray** (bottom-right near the clock): you'll see the thick red crosshair logo with white outline!
2. **Double-click the tray icon** (or right-click ➔ **`⚙ Settings & Sliders...`**) to open the control panel.
3. Switch styles, move sliders live, or drag images directly into the crosshairs folder!
