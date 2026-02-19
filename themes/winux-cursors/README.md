# Winux Cursors

Modern minimalist cursor theme for Winux Linux distribution.

## Design

- **Style**: Clean, minimalist design inspired by macOS but with unique Winux identity
- **Accent Color**: Cyan (#00d4ff)
- **Features**: Subtle drop shadows, smooth animations, high contrast

## Sizes

The theme supports multiple DPI scaling:

| Scale | Size | Directory |
|-------|------|-----------|
| 100%  | 24px | x1        |
| 125%  | 32px | x1.25     |
| 150%  | 48px | x1.5      |
| 200%  | 64px | x2        |

## Cursors Included

### Static Cursors
- `default` - Standard arrow pointer
- `pointer` - Hand cursor for links
- `text` - I-beam for text selection
- `help` - Arrow with question mark
- `crosshair` - Precision selection
- `move` - Four-way move arrows
- `not-allowed` - Forbidden action indicator
- `grab` - Open hand for draggable items
- `grabbing` - Closed hand while dragging
- `zoom-in` - Magnifying glass with plus
- `zoom-out` - Magnifying glass with minus
- `col-resize` - Horizontal resize
- `row-resize` - Vertical resize
- `n-resize`, `s-resize`, `e-resize`, `w-resize` - Edge resize
- `ne-resize`, `nw-resize`, `se-resize`, `sw-resize` - Corner resize
- `all-scroll` - Omni-directional scroll

### Animated Cursors
- `wait` - Loading spinner (12 frames)
- `progress` - Arrow with loading spinner (12 frames)

## Building

### Prerequisites

```bash
# Install required packages
sudo apt install x11-apps python3-pip

# Install Python dependencies
pip3 install Pillow
```

### Build Commands

```bash
# Generate images and build cursors
./build-cursors.sh --generate-images

# Build cursors only (uses existing images)
./build-cursors.sh

# Clean and rebuild
./build-cursors.sh --clean --generate-images
```

## Installation

### System-wide

```bash
sudo cp -r themes/winux-cursors /usr/share/icons/winux-cursors
```

### Current User Only

```bash
mkdir -p ~/.local/share/icons
cp -r themes/winux-cursors ~/.local/share/icons/winux-cursors
```

### Activation

**GNOME:**
```bash
gsettings set org.gnome.desktop.interface cursor-theme 'Winux Cursors'
gsettings set org.gnome.desktop.interface cursor-size 24
```

**KDE Plasma:**
Settings > Appearance > Cursors > Select "Winux Cursors"

**XFCE:**
Settings > Mouse and Touchpad > Theme > Select "Winux Cursors"

## File Structure

```
winux-cursors/
├── index.theme          # Theme metadata
├── cursors/             # Built X11 cursor files
├── build-cursors.sh     # Build script
├── README.md
└── src/
    ├── generate-cursors.py    # Python image generator
    ├── cursors-design.svg     # Visual design reference
    ├── config/                # xcursorgen config files
    │   ├── default.cursor
    │   ├── pointer.cursor
    │   └── ...
    ├── x1/                    # 24px PNG images
    ├── x1.25/                 # 32px PNG images
    ├── x1.5/                  # 48px PNG images
    └── x2/                    # 64px PNG images
```

## Customization

To customize the accent color, edit `src/generate-cursors.py`:

```python
ACCENT_COLOR = (0, 212, 255)  # RGB for #00d4ff
```

Then rebuild the cursors.

## License

Part of the Winux Linux Distribution project.
