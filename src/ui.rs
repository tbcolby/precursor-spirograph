//! UI rendering for Spirograph Engine.

extern crate alloc;
use alloc::format;
use alloc::string::String;

use gam::{Gam, Point, Line, TextView, TextBounds, GlyphStyle};

use crate::app::{SpirographApp, AppState, MenuField, PRESETS};

const SCREEN_W: i16 = 336;
const HEADER_H: i16 = 30;
const FOOTER_Y: i16 = 490;

fn draw_header(gam: &Gam, title: &str) {
    // Dark header background
    gam.draw_rectangle(Point::new(0, 0), Point::new(SCREEN_W, HEADER_H)).ok();

    let mut tv = TextView::new(
        TextBounds::BoundingBox(Point::new(8, 2), Point::new(SCREEN_W - 8, HEADER_H)),
    );
    tv.style = GlyphStyle::Bold;
    tv.invert = true;
    tv.text = String::from(title);
    gam.post_textview(&mut tv).ok();
}

fn draw_footer(gam: &Gam, text: &str) {
    // Separator line
    gam.draw_line(Line::new(Point::new(0, FOOTER_Y), Point::new(SCREEN_W, FOOTER_Y))).ok();

    let mut tv = TextView::new(
        TextBounds::BoundingBox(Point::new(8, FOOTER_Y + 4), Point::new(SCREEN_W - 8, FOOTER_Y + 40)),
    );
    tv.style = GlyphStyle::Small;
    tv.text = String::from(text);
    gam.post_textview(&mut tv).ok();
}

pub fn draw(gam: &Gam, app: &SpirographApp) {
    gam.clear().ok();

    match app.state {
        AppState::Menu => draw_menu(gam, app),
        AppState::Drawing => draw_curve(gam, app),
        AppState::Gallery => draw_gallery(gam, app),
    }

    gam.redraw().ok();
}

fn draw_menu(gam: &Gam, app: &SpirographApp) {
    draw_header(gam, "Spirograph Engine");

    let fields = [
        (MenuField::CurveType, format!("Type: < {} >", app.curve_type.label())),
        (MenuField::BigR, format!("Outer R: < {} >", app.big_r)),
        (MenuField::SmallR, format!("Inner r: < {} >", app.small_r)),
        (MenuField::PenD, format!("Pen d:   < {} >", app.pen_d)),
        (MenuField::Draw, String::from("[ Draw Curve ]")),
        (MenuField::Gallery, String::from("[ Gallery ]")),
    ];

    let start_y: i16 = 60;
    let line_h: i16 = 36;

    for (i, (field, label)) in fields.iter().enumerate() {
        let y = start_y + (i as i16) * line_h;
        let selected = app.menu_field == *field;

        if selected {
            // Highlight bar
            gam.draw_rectangle(
                Point::new(4, y - 2),
                Point::new(SCREEN_W - 4, y + line_h - 6),
            ).ok();
        }

        let mut tv = TextView::new(
            TextBounds::BoundingBox(
                Point::new(12, y),
                Point::new(SCREEN_W - 12, y + line_h),
            ),
        );
        tv.style = GlyphStyle::Regular;
        if selected {
            tv.invert = true;
        }
        tv.text = label.clone();
        gam.post_textview(&mut tv).ok();
    }

    // Info text
    let info_y = start_y + 6 * line_h + 20;
    let mut tv = TextView::new(
        TextBounds::BoundingBox(
            Point::new(12, info_y),
            Point::new(SCREEN_W - 12, info_y + 80),
        ),
    );
    tv.style = GlyphStyle::Small;
    tv.text = String::from(
        "Use arrow keys to navigate and adjust.\nEnter to draw or open gallery.\nMenu key to quit."
    );
    gam.post_textview(&mut tv).ok();

    draw_footer(gam, "Spirograph Engine v0.1");
}

fn draw_curve(gam: &Gam, app: &SpirographApp) {
    let title = format!("{} R={} r={} d={}",
        app.curve_type.label(), app.big_r, app.small_r, app.pen_d);
    draw_header(gam, &title);

    // Draw the curve as connected line segments
    if app.points.len() >= 2 {
        for i in 1..app.points.len() {
            let (x0, y0) = app.points[i - 1];
            let (x1, y1) = app.points[i];

            // Clip to content area
            if y0 >= HEADER_H && y1 >= HEADER_H && y0 < FOOTER_Y && y1 < FOOTER_Y
               && x0 >= 0 && x1 >= 0 && x0 < SCREEN_W && x1 < SCREEN_W
            {
                gam.draw_line(Line::new(
                    Point::new(x0, y0),
                    Point::new(x1, y1),
                )).ok();
            }
        }
    }

    draw_footer(gam, "Enter/Left/Menu: back to menu");
}

fn draw_gallery(gam: &Gam, app: &SpirographApp) {
    draw_header(gam, "Gallery — Presets");

    let start_y: i16 = 40;
    let line_h: i16 = 32;
    let visible = 12; // max visible items

    // Scroll offset
    let scroll = if app.gallery_cursor >= visible {
        app.gallery_cursor - visible + 1
    } else {
        0
    };

    for i in 0..PRESETS.len().min(visible) {
        let idx = scroll + i;
        if idx >= PRESETS.len() { break; }

        let preset = &PRESETS[idx];
        let y = start_y + (i as i16) * line_h;
        let selected = idx == app.gallery_cursor;

        if selected {
            gam.draw_rectangle(
                Point::new(4, y - 2),
                Point::new(SCREEN_W - 4, y + line_h - 6),
            ).ok();
        }

        let label = format!("{} (R={} r={} d={})",
            preset.name, preset.big_r, preset.small_r, preset.pen_d);

        let mut tv = TextView::new(
            TextBounds::BoundingBox(
                Point::new(12, y),
                Point::new(SCREEN_W - 12, y + line_h),
            ),
        );
        tv.style = GlyphStyle::Small;
        if selected {
            tv.invert = true;
        }
        tv.text = label;
        gam.post_textview(&mut tv).ok();
    }

    draw_footer(gam, "Up/Down: browse  Enter: draw  Left: back");
}
