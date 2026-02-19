#!/usr/bin/env python3
"""
Winux Cursor Theme Generator
Generates modern minimalist cursor images with cyan accent color.
Style: Similar to macOS but with unique Winux identity.
"""

import os
import math
from PIL import Image, ImageDraw, ImageFilter, ImageFont

# Configuration
ACCENT_COLOR = (0, 212, 255)  # #00d4ff - Cyan
ACCENT_COLOR_DARK = (0, 170, 204)  # Darker cyan for depth
WHITE = (255, 255, 255)
BLACK = (0, 0, 0)
SHADOW_COLOR = (0, 0, 0, 80)  # Semi-transparent black for shadow
SIZES = {
    'x1': 24,
    'x1.25': 32,
    'x1.5': 48,
    'x2': 64
}

# Output directory
SCRIPT_DIR = os.path.dirname(os.path.abspath(__file__))
OUTPUT_BASE = SCRIPT_DIR


def create_shadow(image, offset=(2, 2), blur_radius=3):
    """Add a subtle drop shadow to an image."""
    # Create shadow layer
    shadow = Image.new('RGBA', image.size, (0, 0, 0, 0))
    shadow_draw = ImageDraw.Draw(shadow)

    # Get alpha channel from original
    alpha = image.split()[3]

    # Create shadow from alpha
    shadow.paste((0, 0, 0, 60), mask=alpha)

    # Blur the shadow
    shadow = shadow.filter(ImageFilter.GaussianBlur(blur_radius))

    # Create new image with shadow
    result = Image.new('RGBA', image.size, (0, 0, 0, 0))

    # Paste shadow with offset (clipped to image bounds)
    result.paste(shadow, offset, shadow)

    # Paste original on top
    result.paste(image, (0, 0), image)

    return result


def draw_rounded_polygon(draw, points, fill, outline=None, outline_width=1, radius=2):
    """Draw a polygon with slightly rounded corners."""
    # For simplicity, draw regular polygon - the anti-aliasing will help
    draw.polygon(points, fill=fill, outline=outline)


def generate_default(size):
    """Generate default arrow cursor."""
    img = Image.new('RGBA', (size, size), (0, 0, 0, 0))
    draw = ImageDraw.Draw(img)

    # Scale factor
    s = size / 24

    # Arrow points - modern sleek design
    arrow_points = [
        (int(3*s), int(3*s)),      # Top point
        (int(3*s), int(19*s)),     # Bottom left
        (int(7*s), int(15*s)),     # Inner corner
        (int(11*s), int(20*s)),    # Bottom right tail
        (int(13*s), int(18*s)),    # Tail outer
        (int(9*s), int(13*s)),     # Inner right
        (int(15*s), int(13*s)),    # Right point
    ]

    # Draw white fill with black outline
    draw.polygon(arrow_points, fill=WHITE, outline=BLACK)

    # Add subtle accent highlight on edge
    highlight_points = [
        (int(3*s), int(3*s)),
        (int(3*s), int(10*s)),
    ]
    draw.line(highlight_points, fill=ACCENT_COLOR, width=max(1, int(s)))

    return create_shadow(img, (int(1*s), int(1*s)), int(2*s))


def generate_pointer(size):
    """Generate hand pointer cursor for links."""
    img = Image.new('RGBA', (size, size), (0, 0, 0, 0))
    draw = ImageDraw.Draw(img)

    s = size / 24

    # Hand pointer - pointing finger
    # Palm
    draw.rounded_rectangle(
        [int(4*s), int(10*s), int(18*s), int(22*s)],
        radius=int(3*s),
        fill=WHITE,
        outline=BLACK
    )

    # Index finger (pointing up)
    draw.rounded_rectangle(
        [int(7*s), int(2*s), int(12*s), int(12*s)],
        radius=int(2*s),
        fill=WHITE,
        outline=BLACK
    )

    # Other fingers (curled)
    for i, x in enumerate([13, 15, 17]):
        draw.rounded_rectangle(
            [int((x-1)*s), int(10*s), int((x+2)*s), int(16*s)],
            radius=int(1*s),
            fill=WHITE,
            outline=BLACK
        )

    # Thumb
    draw.rounded_rectangle(
        [int(2*s), int(12*s), int(6*s), int(17*s)],
        radius=int(1*s),
        fill=WHITE,
        outline=BLACK
    )

    # Accent color on fingertip
    draw.ellipse(
        [int(8*s), int(2*s), int(11*s), int(5*s)],
        fill=ACCENT_COLOR,
        outline=ACCENT_COLOR_DARK
    )

    return create_shadow(img, (int(1*s), int(1*s)), int(2*s))


def generate_text(size):
    """Generate I-beam text cursor."""
    img = Image.new('RGBA', (size, size), (0, 0, 0, 0))
    draw = ImageDraw.Draw(img)

    s = size / 24
    center = size // 2

    # I-beam shape
    beam_width = max(2, int(2*s))
    serif_width = int(6*s)
    serif_height = max(2, int(2*s))

    # Vertical bar
    draw.rectangle(
        [center - beam_width//2, int(4*s), center + beam_width//2, int(20*s)],
        fill=BLACK
    )

    # Top serif
    draw.rectangle(
        [center - serif_width//2, int(3*s), center + serif_width//2, int(3*s) + serif_height],
        fill=BLACK
    )

    # Bottom serif
    draw.rectangle(
        [center - serif_width//2, int(20*s), center + serif_width//2, int(20*s) + serif_height],
        fill=BLACK
    )

    # Accent color highlight in center
    draw.rectangle(
        [center - beam_width//2, center - int(2*s), center + beam_width//2, center + int(2*s)],
        fill=ACCENT_COLOR
    )

    return img


def generate_wait_frame(size, frame):
    """Generate a single frame of the wait spinner."""
    img = Image.new('RGBA', (size, size), (0, 0, 0, 0))
    draw = ImageDraw.Draw(img)

    s = size / 24
    center = size // 2
    radius = int(8*s)
    dot_radius = max(2, int(1.5*s))
    num_dots = 12

    for i in range(num_dots):
        angle = (2 * math.pi * i / num_dots) - math.pi/2
        x = center + int(radius * math.cos(angle))
        y = center + int(radius * math.sin(angle))

        # Calculate opacity based on position relative to current frame
        offset = (i - frame) % num_dots
        opacity = int(255 * (1 - offset / num_dots))

        # Gradient from accent color to gray
        r = int(ACCENT_COLOR[0] * opacity/255 + 100 * (1 - opacity/255))
        g = int(ACCENT_COLOR[1] * opacity/255 + 100 * (1 - opacity/255))
        b = int(ACCENT_COLOR[2] * opacity/255 + 100 * (1 - opacity/255))

        draw.ellipse(
            [x - dot_radius, y - dot_radius, x + dot_radius, y + dot_radius],
            fill=(r, g, b, max(50, opacity))
        )

    return img


def generate_progress_frame(size, frame):
    """Generate a single frame of progress cursor (arrow + spinner)."""
    # Start with default arrow
    img = generate_default(size)

    s = size / 24

    # Add small spinner in bottom right
    spinner_center_x = int(16*s)
    spinner_center_y = int(16*s)
    spinner_radius = int(4*s)
    dot_radius = max(1, int(0.8*s))
    num_dots = 8

    draw = ImageDraw.Draw(img)

    for i in range(num_dots):
        angle = (2 * math.pi * i / num_dots) - math.pi/2
        x = spinner_center_x + int(spinner_radius * math.cos(angle))
        y = spinner_center_y + int(spinner_radius * math.sin(angle))

        offset = (i - frame) % num_dots
        opacity = int(255 * (1 - offset / num_dots))

        r = int(ACCENT_COLOR[0] * opacity/255 + 80 * (1 - opacity/255))
        g = int(ACCENT_COLOR[1] * opacity/255 + 80 * (1 - opacity/255))
        b = int(ACCENT_COLOR[2] * opacity/255 + 80 * (1 - opacity/255))

        draw.ellipse(
            [x - dot_radius, y - dot_radius, x + dot_radius, y + dot_radius],
            fill=(r, g, b, max(80, opacity))
        )

    return img


def generate_help(size):
    """Generate help cursor (arrow + question mark)."""
    img = generate_default(size)
    draw = ImageDraw.Draw(img)

    s = size / 24

    # Question mark circle background
    qm_center_x = int(16*s)
    qm_center_y = int(16*s)
    qm_radius = int(5*s)

    draw.ellipse(
        [qm_center_x - qm_radius, qm_center_y - qm_radius,
         qm_center_x + qm_radius, qm_center_y + qm_radius],
        fill=ACCENT_COLOR,
        outline=ACCENT_COLOR_DARK
    )

    # Question mark
    draw.text(
        (qm_center_x - int(2*s), qm_center_y - int(4*s)),
        "?",
        fill=WHITE,
        font=None
    )

    return img


def generate_crosshair(size):
    """Generate crosshair cursor."""
    img = Image.new('RGBA', (size, size), (0, 0, 0, 0))
    draw = ImageDraw.Draw(img)

    s = size / 24
    center = size // 2
    line_length = int(8*s)
    line_width = max(2, int(2*s))
    gap = int(3*s)

    # Horizontal lines
    draw.rectangle(
        [center - line_length, center - line_width//2,
         center - gap, center + line_width//2],
        fill=BLACK
    )
    draw.rectangle(
        [center + gap, center - line_width//2,
         center + line_length, center + line_width//2],
        fill=BLACK
    )

    # Vertical lines
    draw.rectangle(
        [center - line_width//2, center - line_length,
         center + line_width//2, center - gap],
        fill=BLACK
    )
    draw.rectangle(
        [center - line_width//2, center + gap,
         center + line_width//2, center + line_length],
        fill=BLACK
    )

    # Center dot with accent color
    dot_radius = max(2, int(1.5*s))
    draw.ellipse(
        [center - dot_radius, center - dot_radius,
         center + dot_radius, center + dot_radius],
        fill=ACCENT_COLOR,
        outline=ACCENT_COLOR_DARK
    )

    return img


def generate_move(size):
    """Generate move cursor (4 arrows)."""
    img = Image.new('RGBA', (size, size), (0, 0, 0, 0))
    draw = ImageDraw.Draw(img)

    s = size / 24
    center = size // 2
    arrow_size = int(4*s)
    line_length = int(6*s)

    # Draw 4 arrows pointing outward
    directions = [
        (0, -1),  # Up
        (0, 1),   # Down
        (-1, 0),  # Left
        (1, 0),   # Right
    ]

    for dx, dy in directions:
        # Line from center
        line_end_x = center + dx * line_length
        line_end_y = center + dy * line_length

        draw.line(
            [(center, center), (line_end_x, line_end_y)],
            fill=BLACK,
            width=max(2, int(2*s))
        )

        # Arrow head
        arrow_tip_x = center + dx * (line_length + arrow_size)
        arrow_tip_y = center + dy * (line_length + arrow_size)

        if dx != 0:  # Horizontal
            points = [
                (arrow_tip_x, arrow_tip_y),
                (line_end_x, line_end_y - arrow_size//2),
                (line_end_x, line_end_y + arrow_size//2),
            ]
        else:  # Vertical
            points = [
                (arrow_tip_x, arrow_tip_y),
                (line_end_x - arrow_size//2, line_end_y),
                (line_end_x + arrow_size//2, line_end_y),
            ]

        draw.polygon(points, fill=BLACK)

    # Center accent circle
    dot_radius = max(2, int(2*s))
    draw.ellipse(
        [center - dot_radius, center - dot_radius,
         center + dot_radius, center + dot_radius],
        fill=ACCENT_COLOR,
        outline=ACCENT_COLOR_DARK
    )

    return img


def generate_not_allowed(size):
    """Generate not-allowed cursor (circle with line)."""
    img = Image.new('RGBA', (size, size), (0, 0, 0, 0))
    draw = ImageDraw.Draw(img)

    s = size / 24
    center = size // 2
    radius = int(9*s)
    line_width = max(2, int(2.5*s))

    # Red circle
    RED = (220, 53, 69)  # Bootstrap red
    RED_DARK = (185, 43, 58)

    draw.ellipse(
        [center - radius, center - radius,
         center + radius, center + radius],
        outline=RED,
        width=line_width
    )

    # Diagonal line
    offset = int(radius * 0.7)
    draw.line(
        [(center - offset, center - offset),
         (center + offset, center + offset)],
        fill=RED,
        width=line_width
    )

    return img


def generate_grab(size):
    """Generate open hand grab cursor."""
    img = Image.new('RGBA', (size, size), (0, 0, 0, 0))
    draw = ImageDraw.Draw(img)

    s = size / 24

    # Palm
    draw.rounded_rectangle(
        [int(4*s), int(10*s), int(20*s), int(22*s)],
        radius=int(3*s),
        fill=WHITE,
        outline=BLACK
    )

    # Fingers (open, spread)
    finger_positions = [5, 8, 11, 14]
    finger_heights = [4, 2, 2, 4]

    for i, (x, h) in enumerate(zip(finger_positions, finger_heights)):
        draw.rounded_rectangle(
            [int(x*s), int(h*s), int((x+3)*s), int(12*s)],
            radius=int(1*s),
            fill=WHITE,
            outline=BLACK
        )

    # Thumb
    draw.rounded_rectangle(
        [int(17*s), int(8*s), int(21*s), int(14*s)],
        radius=int(1*s),
        fill=WHITE,
        outline=BLACK
    )

    return create_shadow(img, (int(1*s), int(1*s)), int(2*s))


def generate_grabbing(size):
    """Generate closed hand grabbing cursor."""
    img = Image.new('RGBA', (size, size), (0, 0, 0, 0))
    draw = ImageDraw.Draw(img)

    s = size / 24

    # Closed fist
    draw.rounded_rectangle(
        [int(4*s), int(8*s), int(20*s), int(20*s)],
        radius=int(4*s),
        fill=WHITE,
        outline=BLACK
    )

    # Knuckles (bumps on top)
    for x in [6, 10, 14]:
        draw.ellipse(
            [int(x*s), int(6*s), int((x+3)*s), int(10*s)],
            fill=WHITE,
            outline=BLACK
        )

    # Thumb wrapped around
    draw.rounded_rectangle(
        [int(3*s), int(12*s), int(7*s), int(18*s)],
        radius=int(1*s),
        fill=WHITE,
        outline=BLACK
    )

    return create_shadow(img, (int(1*s), int(1*s)), int(2*s))


def generate_zoom_in(size):
    """Generate zoom-in cursor (magnifying glass with +)."""
    img = Image.new('RGBA', (size, size), (0, 0, 0, 0))
    draw = ImageDraw.Draw(img)

    s = size / 24

    # Magnifying glass
    glass_center_x = int(9*s)
    glass_center_y = int(9*s)
    glass_radius = int(6*s)

    # Glass circle
    draw.ellipse(
        [glass_center_x - glass_radius, glass_center_y - glass_radius,
         glass_center_x + glass_radius, glass_center_y + glass_radius],
        fill=WHITE,
        outline=BLACK,
        width=max(2, int(1.5*s))
    )

    # Handle
    handle_start_x = glass_center_x + int(glass_radius * 0.7)
    handle_start_y = glass_center_y + int(glass_radius * 0.7)
    draw.line(
        [(handle_start_x, handle_start_y),
         (int(20*s), int(20*s))],
        fill=BLACK,
        width=max(3, int(2.5*s))
    )

    # Plus sign in glass (accent color)
    plus_size = int(3*s)
    line_width = max(2, int(1.5*s))

    draw.line(
        [(glass_center_x - plus_size, glass_center_y),
         (glass_center_x + plus_size, glass_center_y)],
        fill=ACCENT_COLOR,
        width=line_width
    )
    draw.line(
        [(glass_center_x, glass_center_y - plus_size),
         (glass_center_x, glass_center_y + plus_size)],
        fill=ACCENT_COLOR,
        width=line_width
    )

    return create_shadow(img, (int(1*s), int(1*s)), int(2*s))


def generate_zoom_out(size):
    """Generate zoom-out cursor (magnifying glass with -)."""
    img = Image.new('RGBA', (size, size), (0, 0, 0, 0))
    draw = ImageDraw.Draw(img)

    s = size / 24

    # Magnifying glass
    glass_center_x = int(9*s)
    glass_center_y = int(9*s)
    glass_radius = int(6*s)

    # Glass circle
    draw.ellipse(
        [glass_center_x - glass_radius, glass_center_y - glass_radius,
         glass_center_x + glass_radius, glass_center_y + glass_radius],
        fill=WHITE,
        outline=BLACK,
        width=max(2, int(1.5*s))
    )

    # Handle
    handle_start_x = glass_center_x + int(glass_radius * 0.7)
    handle_start_y = glass_center_y + int(glass_radius * 0.7)
    draw.line(
        [(handle_start_x, handle_start_y),
         (int(20*s), int(20*s))],
        fill=BLACK,
        width=max(3, int(2.5*s))
    )

    # Minus sign in glass (accent color)
    minus_size = int(3*s)
    line_width = max(2, int(1.5*s))

    draw.line(
        [(glass_center_x - minus_size, glass_center_y),
         (glass_center_x + minus_size, glass_center_y)],
        fill=ACCENT_COLOR,
        width=line_width
    )

    return create_shadow(img, (int(1*s), int(1*s)), int(2*s))


def generate_col_resize(size):
    """Generate column resize cursor (horizontal double arrow)."""
    img = Image.new('RGBA', (size, size), (0, 0, 0, 0))
    draw = ImageDraw.Draw(img)

    s = size / 24
    center = size // 2

    # Horizontal double arrow
    arrow_size = int(4*s)
    line_length = int(6*s)
    line_width = max(2, int(2*s))

    # Center line
    draw.rectangle(
        [center - line_length, center - line_width//2,
         center + line_length, center + line_width//2],
        fill=BLACK
    )

    # Left arrow
    draw.polygon([
        (center - line_length - arrow_size, center),
        (center - line_length, center - arrow_size),
        (center - line_length, center + arrow_size),
    ], fill=BLACK)

    # Right arrow
    draw.polygon([
        (center + line_length + arrow_size, center),
        (center + line_length, center - arrow_size),
        (center + line_length, center + arrow_size),
    ], fill=BLACK)

    # Accent dot in center
    dot_radius = max(1, int(1*s))
    draw.ellipse(
        [center - dot_radius, center - dot_radius,
         center + dot_radius, center + dot_radius],
        fill=ACCENT_COLOR
    )

    return img


def generate_row_resize(size):
    """Generate row resize cursor (vertical double arrow)."""
    img = Image.new('RGBA', (size, size), (0, 0, 0, 0))
    draw = ImageDraw.Draw(img)

    s = size / 24
    center = size // 2

    # Vertical double arrow
    arrow_size = int(4*s)
    line_length = int(6*s)
    line_width = max(2, int(2*s))

    # Center line
    draw.rectangle(
        [center - line_width//2, center - line_length,
         center + line_width//2, center + line_length],
        fill=BLACK
    )

    # Top arrow
    draw.polygon([
        (center, center - line_length - arrow_size),
        (center - arrow_size, center - line_length),
        (center + arrow_size, center - line_length),
    ], fill=BLACK)

    # Bottom arrow
    draw.polygon([
        (center, center + line_length + arrow_size),
        (center - arrow_size, center + line_length),
        (center + arrow_size, center + line_length),
    ], fill=BLACK)

    # Accent dot in center
    dot_radius = max(1, int(1*s))
    draw.ellipse(
        [center - dot_radius, center - dot_radius,
         center + dot_radius, center + dot_radius],
        fill=ACCENT_COLOR
    )

    return img


def generate_resize_arrow(size, direction):
    """Generate directional resize arrows."""
    img = Image.new('RGBA', (size, size), (0, 0, 0, 0))
    draw = ImageDraw.Draw(img)

    s = size / 24
    center = size // 2
    arrow_size = int(5*s)
    line_width = max(2, int(2*s))

    # Direction vectors
    directions = {
        'n': (0, -1),
        's': (0, 1),
        'e': (1, 0),
        'w': (-1, 0),
        'ne': (1, -1),
        'nw': (-1, -1),
        'se': (1, 1),
        'sw': (-1, 1),
    }

    dx, dy = directions[direction]

    # Normalize diagonal vectors
    if abs(dx) == 1 and abs(dy) == 1:
        length = int(8*s)
    else:
        length = int(10*s)

    # Calculate endpoints
    x1 = center - dx * length // 2
    y1 = center - dy * length // 2
    x2 = center + dx * length // 2
    y2 = center + dy * length // 2

    # Draw line
    draw.line([(x1, y1), (x2, y2)], fill=BLACK, width=line_width)

    # Draw arrowheads at both ends
    for end_x, end_y, dir_mult in [(x1, y1, -1), (x2, y2, 1)]:
        if direction in ['n', 's']:
            points = [
                (end_x, end_y),
                (end_x - arrow_size, end_y - dir_mult * dy * arrow_size),
                (end_x + arrow_size, end_y - dir_mult * dy * arrow_size),
            ]
        elif direction in ['e', 'w']:
            points = [
                (end_x, end_y),
                (end_x - dir_mult * dx * arrow_size, end_y - arrow_size),
                (end_x - dir_mult * dx * arrow_size, end_y + arrow_size),
            ]
        else:  # Diagonal
            perp_dx, perp_dy = -dy, dx  # Perpendicular vector
            points = [
                (end_x, end_y),
                (int(end_x - dir_mult * dx * arrow_size + perp_dx * arrow_size * 0.5),
                 int(end_y - dir_mult * dy * arrow_size + perp_dy * arrow_size * 0.5)),
                (int(end_x - dir_mult * dx * arrow_size - perp_dx * arrow_size * 0.5),
                 int(end_y - dir_mult * dy * arrow_size - perp_dy * arrow_size * 0.5)),
            ]

        draw.polygon(points, fill=BLACK)

    return img


def generate_all_scroll(size):
    """Generate all-scroll cursor (4-way scroll)."""
    img = Image.new('RGBA', (size, size), (0, 0, 0, 0))
    draw = ImageDraw.Draw(img)

    s = size / 24
    center = size // 2

    # Central circle
    circle_radius = int(4*s)
    draw.ellipse(
        [center - circle_radius, center - circle_radius,
         center + circle_radius, center + circle_radius],
        fill=WHITE,
        outline=BLACK,
        width=max(1, int(1*s))
    )

    # Four small triangular arrows
    arrow_dist = int(7*s)
    arrow_size = int(3*s)

    # Up arrow
    draw.polygon([
        (center, center - arrow_dist - arrow_size),
        (center - arrow_size, center - arrow_dist),
        (center + arrow_size, center - arrow_dist),
    ], fill=BLACK)

    # Down arrow
    draw.polygon([
        (center, center + arrow_dist + arrow_size),
        (center - arrow_size, center + arrow_dist),
        (center + arrow_size, center + arrow_dist),
    ], fill=BLACK)

    # Left arrow
    draw.polygon([
        (center - arrow_dist - arrow_size, center),
        (center - arrow_dist, center - arrow_size),
        (center - arrow_dist, center + arrow_size),
    ], fill=BLACK)

    # Right arrow
    draw.polygon([
        (center + arrow_dist + arrow_size, center),
        (center + arrow_dist, center - arrow_size),
        (center + arrow_dist, center + arrow_size),
    ], fill=BLACK)

    # Accent center dot
    dot_radius = max(1, int(1.5*s))
    draw.ellipse(
        [center - dot_radius, center - dot_radius,
         center + dot_radius, center + dot_radius],
        fill=ACCENT_COLOR
    )

    return img


def main():
    """Generate all cursor images."""
    print("Generating Winux Cursor Theme...")

    # Create output directories
    for size_name in SIZES.keys():
        os.makedirs(os.path.join(OUTPUT_BASE, size_name), exist_ok=True)

    # Static cursors
    static_cursors = {
        'default': generate_default,
        'pointer': generate_pointer,
        'text': generate_text,
        'help': generate_help,
        'crosshair': generate_crosshair,
        'move': generate_move,
        'not-allowed': generate_not_allowed,
        'grab': generate_grab,
        'grabbing': generate_grabbing,
        'zoom-in': generate_zoom_in,
        'zoom-out': generate_zoom_out,
        'col-resize': generate_col_resize,
        'row-resize': generate_row_resize,
        'all-scroll': generate_all_scroll,
    }

    # Generate static cursors
    for cursor_name, generator in static_cursors.items():
        print(f"  Generating {cursor_name}...")
        for size_name, size in SIZES.items():
            img = generator(size)
            output_path = os.path.join(OUTPUT_BASE, size_name, f"{cursor_name}.png")
            img.save(output_path, 'PNG')

    # Resize cursors
    resize_directions = ['n', 's', 'e', 'w', 'ne', 'nw', 'se', 'sw']
    for direction in resize_directions:
        cursor_name = f"{direction}-resize"
        print(f"  Generating {cursor_name}...")
        for size_name, size in SIZES.items():
            img = generate_resize_arrow(size, direction)
            output_path = os.path.join(OUTPUT_BASE, size_name, f"{cursor_name}.png")
            img.save(output_path, 'PNG')

    # Animated cursors
    num_frames = 12

    print("  Generating wait animation...")
    for frame in range(num_frames):
        for size_name, size in SIZES.items():
            img = generate_wait_frame(size, frame)
            output_path = os.path.join(OUTPUT_BASE, size_name, f"wait-{frame+1:02d}.png")
            img.save(output_path, 'PNG')

    print("  Generating progress animation...")
    for frame in range(num_frames):
        for size_name, size in SIZES.items():
            img = generate_progress_frame(size, frame)
            output_path = os.path.join(OUTPUT_BASE, size_name, f"progress-{frame+1:02d}.png")
            img.save(output_path, 'PNG')

    print("Done! Cursor images generated in src/")


if __name__ == '__main__':
    main()
