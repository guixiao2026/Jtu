use egui::Color32;

// Mask overlay
pub const MASK_COLOR: Color32 =
    Color32::from_rgba_premultiplied(0, 0, 0, 128);

// Selection border
pub const SELECTION_BORDER_COLOR: Color32 =
    Color32::from_rgb(64, 150, 255);
pub const SELECTION_BORDER_WIDTH: f32 = 2.0;

// Control points
pub const CONTROL_POINT_RADIUS: f32 = 5.0;
pub const CONTROL_POINT_FILL: Color32 = Color32::WHITE;
pub const CONTROL_POINT_STROKE_COLOR: Color32 =
    Color32::from_rgb(64, 150, 255);
pub const CONTROL_POINT_STROKE_WIDTH: f32 = 1.5;
pub const CONTROL_POINT_GLOW: Color32 =
    Color32::from_rgba_premultiplied(64, 150, 255, 30);

// Size indicator capsule
pub const SIZE_CAPSULE_BG: Color32 =
    Color32::from_rgba_premultiplied(0, 0, 0, 160);
pub const SIZE_CAPSULE_TEXT: Color32 = Color32::WHITE;
pub const SIZE_CAPSULE_FONT_SIZE: f32 = 12.0;
pub const SIZE_CAPSULE_CORNER_RADIUS: f32 = 4.0;
pub const SIZE_CAPSULE_PADDING_H: f32 = 8.0;
pub const SIZE_CAPSULE_PADDING_V: f32 = 3.0;
pub const SIZE_CAPSULE_GAP: f32 = 6.0;

// Toolbar
pub const TOOLBAR_BG: Color32 =
    Color32::from_rgba_premultiplied(30, 30, 30, 240);
pub const TOOLBAR_CORNER_RADIUS: f32 = 8.0;
pub const TOOLBAR_INNER_MARGIN: f32 = 6.0;
pub const TOOLBAR_GLOW_COLOR: Color32 =
    Color32::from_rgba_premultiplied(64, 150, 255, 50);
pub const TOOLBAR_GLOW_BLUR: u8 = 6;

// Toolbar separator
pub const SEPARATOR_COLOR: Color32 =
    Color32::from_rgba_premultiplied(
        255, 255, 255, 40,
    );
pub const SEPARATOR_WIDTH: f32 = 1.0;
pub const SEPARATOR_HEIGHT: f32 = 16.0;

// Toolbar buttons
pub const TOOL_ICON_SIZE: f32 = 18.0;
pub const TOOL_BTN_MIN_SIZE: f32 = 28.0;
pub const TOOL_BTN_CORNER_RADIUS: f32 = 4.0;
pub const TOOL_ACTIVE_TEXT: Color32 =
    Color32::from_rgb(64, 150, 255);
pub const TOOL_ACTIVE_BG: Color32 =
    Color32::from_rgba_premultiplied(64, 150, 255, 40);
pub const TOOL_INACTIVE_TEXT: Color32 =
    Color32::from_rgb(180, 180, 180);
pub const CANCEL_TEXT_COLOR: Color32 =
    Color32::from_rgb(255, 80, 80);
pub const COPY_TEXT_COLOR: Color32 =
    Color32::from_rgb(64, 150, 255);

// Arrow
pub const ARROW_HEAD_LENGTH_FACTOR: f32 = 4.0;
pub const ARROW_HEAD_MIN_LENGTH: f32 = 14.0;
pub const ARROW_HEAD_WIDTH_RATIO: f32 = 0.4;

// Blur
pub const BLUR_BLOCK_SIZE: f32 = 10.0;

// Color picker
pub const COLOR_PICKER_COLORS: [Color32; 8] = [
    Color32::from_rgb(255, 59, 48),  // Red
    Color32::from_rgb(255, 149, 0),  // Orange
    Color32::from_rgb(255, 204, 0),  // Yellow
    Color32::from_rgb(52, 199, 89), // Green
    Color32::from_rgb(64, 150, 255), // Blue
    Color32::from_rgb(175, 82, 222), // Purple
    Color32::WHITE,
    Color32::BLACK,
];
pub const COLOR_SWATCH_RADIUS: f32 = 9.0;
pub const COLOR_SWATCH_SELECTED_STROKE: f32 = 2.0;

// Text input
pub const TEXT_INPUT_BG: Color32 =
    Color32::from_rgba_premultiplied(30, 30, 30, 230);
pub const TEXT_INPUT_CORNER_RADIUS: f32 = 6.0;

// Main window
pub const TITLE_FONT_SIZE: f32 = 28.0;
pub const SUBTITLE_FONT_SIZE: f32 = 13.0;
pub const SUBTITLE_COLOR: Color32 =
    Color32::from_rgb(140, 140, 140);
pub const MAIN_BTN_WIDTH: f32 = 200.0;
pub const MAIN_BTN_HEIGHT: f32 = 40.0;
pub const MAIN_BTN_CORNER_RADIUS: f32 = 8.0;
pub const SHORTCUT_HINT_COLOR: Color32 =
    Color32::from_rgb(100, 100, 100);

// Latency timer
pub const LATENCY_TIMER_OFFSET_X: f32 = 120.0;
pub const LATENCY_TIMER_OFFSET_Y: f32 = 8.0;

// Magnifier
pub const MAGNIFIER_SIZE: f32 = 120.0;
pub const MAGNIFIER_ZOOM: f32 = 8.0;
pub const MAGNIFIER_OFFSET_Y: f32 = 20.0;
pub const MAGNIFIER_BORDER_COLOR: Color32 =
    Color32::from_rgb(64, 150, 255);
pub const MAGNIFIER_BORDER_WIDTH: f32 = 2.0;
pub const MAGNIFIER_CROSSHAIR_COLOR: Color32 =
    Color32::from_rgba_premultiplied(64, 150, 255, 180);
pub const MAGNIFIER_BG: Color32 =
    Color32::from_rgba_premultiplied(0, 0, 0, 220);
pub const MAGNIFIER_INFO_FONT_SIZE: f32 = 11.0;
pub const MAGNIFIER_INFO_HEIGHT: f32 = 36.0;
pub const MAGNIFIER_COLOR_SWATCH: f32 = 14.0;
