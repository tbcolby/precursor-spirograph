//! Spirograph shell: keys -> core operations, drawing, nothing else. Curve
//! maths, presets and the incremental pen are in `spirograph-core`.

use gam::GlyphStyle;
use precursor_appkit::keys::Key;
use precursor_appkit::layout::ListState;
use precursor_appkit::screen::{Screen, MARGIN};
use spirograph_core::{Curve, Params, Pen, PRESETS, SEGMENTS_PER_TICK};

#[derive(PartialEq, Eq)]
pub enum Action {
    Stay,
    Leave,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum View {
    Menu,
    Drawing,
    Gallery,
}

const MENU_ROWS: usize = 6;
const ROW_DRAW: usize = 4;
const ROW_GALLERY: usize = 5;

pub struct App {
    screen: Screen,
    view: View,
    params: Params,
    menu: ListState,
    gallery: ListState,
    visible: usize,
    pen: Pen,
}

impl App {
    pub fn new(screen: Screen) -> Self {
        let params = PRESETS[0].params;
        let mut app = Self {
            screen,
            view: View::Menu,
            params,
            menu: ListState::default(),
            gallery: ListState::default(),
            visible: 1,
            pen: Pen::new(Curve::new(params, (0, 0), 1)),
        };
        app.restart();
        app
    }

    pub fn screen(&self) -> &Screen { &self.screen }

    /// True while the pen is still moving (the pump runs only then).
    pub fn animating(&self) -> bool { self.view == View::Drawing && !self.pen.done() }

    /// Content area between header and footer, matching `Screen::header`/`footer`.
    fn content(&self) -> (isize, isize) {
        let top = 2 + self.screen.line_height(GlyphStyle::Bold) + 2 + 4;
        let bottom = self.screen.size().y - self.screen.line_height(GlyphStyle::Small) - 2 - 4;
        (top, bottom)
    }

    fn clip(&self) -> ((i32, i32), (i32, i32)) {
        let (top, bottom) = self.content();
        ((0, top as i32), (self.screen.size().x as i32 - 1, bottom as i32))
    }

    fn restart(&mut self) {
        let (top, bottom) = self.content();
        let w = self.screen.size().x as i32;
        let fit = ((w - 2 * MARGIN as i32) / 2).min((bottom - top) as i32 / 2 - 2).max(8);
        self.pen = Pen::new(Curve::new(self.params, (w / 2, ((top + bottom) / 2) as i32), fit));
    }

    fn start(&mut self, p: Params) {
        self.params = p;
        self.restart();
        self.view = View::Drawing;
    }

    pub fn on_key(&mut self, key: Key) -> Action {
        match self.view {
            View::Menu => return self.menu_key(key),
            View::Drawing => match key {
                Key::Enter => self.restart(),
                k if k.is_back() => self.view = View::Menu,
                _ => {}
            },
            View::Gallery => match key {
                Key::Up => self.gallery.step(-1, PRESETS.len(), self.visible),
                Key::Down => self.gallery.step(1, PRESETS.len(), self.visible),
                Key::Enter => self.start(PRESETS[self.gallery.selected].params),
                k if k.is_back() => self.view = View::Menu,
                _ => {}
            },
        }
        Action::Stay
    }

    fn menu_key(&mut self, key: Key) -> Action {
        match key {
            Key::Up => self.menu.step_wrapping(-1, MENU_ROWS, self.visible),
            Key::Down => self.menu.step_wrapping(1, MENU_ROWS, self.visible),
            Key::Left | Key::Right => {
                let d = if key == Key::Left { -1 } else { 1 };
                match self.menu.selected {
                    0 => self.params.toggle_kind(),
                    1 => self.params.step_big_r(d),
                    2 => self.params.step_small_r(d),
                    3 => self.params.step_pen_d(d),
                    _ => {}
                }
            }
            Key::Enter => match self.menu.selected {
                0 => self.params.toggle_kind(),
                ROW_DRAW => self.start(self.params),
                ROW_GALLERY => self.view = View::Gallery,
                _ => {}
            },
            k if k.is_back() => return Action::Leave,
            _ => {}
        }
        Action::Stay
    }

    pub fn on_foreground(&mut self) {
        self.screen.refresh_size();
        if self.view == View::Drawing {
            self.restart();
        }
    }

    /// Reveal the next segments; only those are drawn (no clear, no redraw).
    pub fn on_tick(&mut self) {
        if self.view != View::Drawing {
            return;
        }
        let range = self.pen.advance(SEGMENTS_PER_TICK);
        self.screen.lines(self.pen.curve.segments(range), self.clip(), 1);
        self.screen.flush();
    }

    pub fn draw(&mut self) {
        self.screen.clear();
        let p = self.params;
        match self.view {
            View::Menu => {
                let top = self.screen.header("Spirograph");
                let bottom = self.screen.footer("↑↓ field  ←→ change  ⏎ go  ⌫ quit");
                let items = [
                    format!("Type: {}", p.kind.label()),
                    format!("Outer R: {}", p.big_r),
                    format!("Inner r: {}", p.small_r),
                    format!("Pen d: {}", p.pen_d),
                    "Draw".to_string(),
                    "Gallery".to_string(),
                ];
                self.visible = self.screen.list(top, bottom, &items, &self.menu);
            }
            View::Drawing => {
                self.screen.header(&format!("R{} r{} d{}  {} turns", p.big_r, p.small_r, p.pen_d, p.period_turns()));
                self.screen.footer("⏎ redraw  ⌫ back");
                let drawn = self.pen.drawn();
                self.screen.lines(self.pen.curve.segments(0..drawn), self.clip(), 1);
            }
            View::Gallery => {
                let top = self.screen.header("Gallery");
                let bottom = self.screen.footer("↑↓ move  ⏎ draw  ⌫ back");
                let items: Vec<&str> = PRESETS.iter().map(|pr| pr.name).collect();
                self.visible = self.screen.list(top, bottom, &items, &self.gallery);
            }
        }
        self.screen.flush();
    }
}
