use sdl2::pixels::Color;

use crate::screen::Screen;

pub trait SdlFontRendererExt {
    fn render_font_string_wrapped(
        &mut self,
        x: i32, y: i32,
        right_bound: u32,
        texture: &mut sdl2::render::Texture,
        font: &crate::font::Font,
        string: &str
    ) -> Result<(i32, i32), String>;
}

impl SdlFontRendererExt for sdl2::render::WindowCanvas {
    fn render_font_string_wrapped(&mut self, x: i32, y: i32, right_bound: u32, texture: &mut sdl2::render::Texture, font: &crate::font::Font, string: &str) -> Result<(i32, i32), String> {
        let previous_color_mod = texture.color_mod();
        let previous_alpha = texture.alpha_mod();
        let canvas_color = self.draw_color();

        texture.set_alpha_mod(canvas_color.a);
        texture.set_color_mod(canvas_color.r, canvas_color.g, canvas_color.b);

        let mut mutx = x;
        let mut muty = y;

        for glyph in string.chars() {
            // Check if we should wrap over the screen
            match glyph {
                '\r' => (),
                '\n' => {
                    muty += font.glyph_size.1 as i32;
                    mutx = x;
                },
                '\t' => {
                    mutx += font.glyph_size.0 as i32 * 4;
                },
                _ => {
                    let rect = font.get_glyph_rect_sdl(glyph);
                    let dest = sdl2::rect::Rect::new(mutx, muty, rect.width(), rect.height());
                    self.copy(texture, Some(rect), Some(dest))?;
                    mutx += font.glyph_size.0 as i32;
                }
            }
            if (mutx+font.glyph_size.0 as i32) > (right_bound as i32) {
                muty += font.glyph_size.1 as i32;
                mutx = x;
            }
        }

        texture.set_alpha_mod(previous_alpha);
        texture.set_color_mod(previous_color_mod.0, previous_color_mod.1, previous_color_mod.2);

        Ok((mutx, muty))
    }
}

const MOD13_PAL: [Color; 16] = [
    Color::RGB(0, 0, 0),
    Color::RGB(0, 0, 0xaa),
    Color::RGB(0, 0xaa, 0),
    Color::RGB(0, 0xaa, 0xaa),
    Color::RGB(0xaa, 0, 0),
    Color::RGB(0xaa, 0, 0xaa),
    Color::RGB(0xaa, 0x55, 0),
    Color::RGB(0xaa, 0xaa, 0xaa),
    Color::RGB(0x55, 0x55, 0x55),
    Color::RGB(0x55, 0x55, 0xff),
    Color::RGB(0x55, 0xff, 0x55),
    Color::RGB(0x55, 0xff, 0xff),
    Color::RGB(0xff, 0x55, 0x55),
    Color::RGB(0xff, 0x55, 0xff),
    Color::RGB(0xff, 0xff, 0x55),
    Color::RGB(0xff, 0xff, 0xff)
];

enum ScrollbarState {
    Blurred,
    Hovered,
    Pressed
}

pub struct VisualCommandLine<'a> {
    scroll_locked: bool,
    scrollbar_state: ScrollbarState,
    font: crate::font::Font,
    font_texture: sdl2::render::Texture<'a>,
    ticks: usize,
    last_pos: (u32, u32),
    scroll: u32
}

impl<'a> VisualCommandLine<'a> {
    pub fn set_font_texture(&mut self, tex: sdl2::render::Texture<'a>) {
        self.font_texture = tex;
    }

    pub fn new(font_texture: sdl2::render::Texture<'a>, font: crate::font::Font) -> Self {
        Self { ticks: 0, font_texture, font, scroll: 0, scrollbar_state: ScrollbarState::Blurred, last_pos: (0, 0), scroll_locked: true }
    }

    fn is_caret_rendered(&self) -> bool {
        (self.ticks / 10) % 2 == 0
    }

    pub fn mouse_press(&mut self, canvas: &sdl2::render::WindowCanvas, mouse_pos: (i32, i32)) {
        let height = canvas.window().size().1 as i32;
        let width = canvas.window().size().0 as i32;
        let top_half_rect = sdl2::rect::Rect::new(width-16, 0, 16, 16);
        let bottom_half_rect = sdl2::rect::Rect::new(width-16, height-16, 16, 16);
        let overflow_height = self.last_pos.1;
        let scroll_rect = self.get_scrollbar_thumb_rect(canvas, overflow_height as u32);

        if scroll_rect.contains_point((mouse_pos.0, mouse_pos.1)) {
            self.scrollbar_state = ScrollbarState::Pressed;
            self.scroll_locked = false;
        }
        if top_half_rect.contains_point((mouse_pos.0, mouse_pos.1)) {
            self.scroll = (self.scroll as i32 - 20).max(0) as u32;
            self.scroll_locked = false;
        }
        if bottom_half_rect.contains_point((mouse_pos.0, mouse_pos.1)) {
            self.scroll += 20;
            self.scroll_locked = false;
        }
    }

    pub fn mouse_move(&mut self, canvas: &sdl2::render::WindowCanvas, mouse_pos: (i32, i32), mouse_delta_y: i32) -> bool {
        let overflow_height = self.last_pos.1;
        let scroll_rect = self.get_scrollbar_thumb_rect(canvas, overflow_height as u32);
        let height = canvas.window().size().1 as i32;
        let thumb_height = height*height/(overflow_height as i32).max(1);

        if let ScrollbarState::Pressed = self.scrollbar_state {
            self.scroll = (self.scroll as i32 + (mouse_delta_y * overflow_height as i32 / (height-17*2))).max(0) as u32;

            return true;
        }
        else if scroll_rect.contains_point((mouse_pos.0, mouse_pos.1)) {
            self.scrollbar_state = ScrollbarState::Hovered;
        }
        else {
             self.scrollbar_state = ScrollbarState::Blurred;       
        }
        return false;
    }

    pub fn lock_scroll(&mut self) {
        self.scroll_locked = true;
    }

    pub fn mouse_release(&mut self, canvas: &sdl2::render::WindowCanvas, mouse_pos: (i32, i32)) {
        self.scrollbar_state = ScrollbarState::Blurred;
    }

    pub fn get_scrollbar_thumb_rect(&self, canvas: &sdl2::render::WindowCanvas, overflow_height: u32) -> sdl2::rect::Rect {
        // We subtract 17 because of lower arrow
        let mut height = canvas.window().size().1.max(17*2);
        if height > overflow_height {
            return sdl2::rect::Rect::new(0, 0, 0, 0);
        }

        let scroll = self.scroll.min(overflow_height-height);
        let starting_at = canvas.window().size().0-15;
        let thumb_height = (height-17*2)*height/(overflow_height);
        sdl2::rect::Rect::new(starting_at as i32, (scroll*(height-17*2)/overflow_height) as i32 + 17, 14, thumb_height)
    }

    fn render_embedded_bitmap(&self, canvas: &mut sdl2::render::WindowCanvas, buf: &[&str], sx: i32, sy: i32) {
        for (y, row) in buf.iter().enumerate() {
            for (x, chr) in (*row).chars().enumerate() {
                if chr == '#' {
                    canvas.draw_point((sx+x as i32, sy+y as i32)).unwrap();
                }
            }
        }
    }

    pub fn scroll_by(&mut self, n: i32) {
        self.scroll_locked = false;
        self.scroll = (self.scroll as i32 + n).max(0) as u32;
    }

    pub fn render_scrollbar(&self, canvas: &mut sdl2::render::WindowCanvas) {
        let overflow_height = self.last_pos.1;
        let height = canvas.window().size().1.max(17*2);
        // No reason to render scroll bar if overflow_height is less than height
        if height > overflow_height {
            return;
        }
        
        let starting_at = canvas.window().size().0-16;

        // Windows 10-style scrollbar (hardcoded)
        // Here we render the thin white line between scrollbar and cmd
        canvas.set_draw_color(Color::WHITE);
        canvas.fill_rect(sdl2::rect::Rect::new(starting_at as i32, 0, 1, height)).unwrap();

        // Scrollbar background
        canvas.set_draw_color(Color::RGB(0xf0, 0xf0, 0xf0));
        canvas.fill_rect(sdl2::rect::Rect::new(starting_at as i32 + 1, 0, 15, height)).unwrap();

        // Scrollbar thumb
        match self.scrollbar_state {
            ScrollbarState::Blurred => 
                canvas.set_draw_color(Color::RGB(0xcd, 0xcd, 0xcd)),
            ScrollbarState::Hovered => 
                canvas.set_draw_color(Color::RGB(0xab, 0xab, 0xab)),
            ScrollbarState::Pressed => 
                canvas.set_draw_color(Color::RGB(0x9a, 0x9a, 0x9a)),
        }
        canvas.fill_rect(self.get_scrollbar_thumb_rect(canvas, overflow_height)).unwrap();

        // Render top arrow
        canvas.set_draw_color(Color::RGB(0x60, 0x60, 0x60));
        let mut arrow_b = vec!["   #   ",
                           "  ###  ",
                           " ##### ",
                           "### ###",
                           "##   ##",
                           "#     #"];
 
        self.render_embedded_bitmap(canvas, &arrow_b, starting_at as i32 + 4, 6);
        
        // This is the worst hack
        arrow_b.reverse();

        self.render_embedded_bitmap(canvas, &arrow_b, starting_at as i32 + 4, height as i32-17+6);       
    }

    pub fn render(&mut self, canvas: &mut sdl2::render::WindowCanvas, cmd: &Screen) {
        let (width, height) = canvas.window().size();
        // Background
        canvas.set_draw_color(MOD13_PAL[((cmd.color>>4)&0xF) as usize]);
        canvas.clear();

        // Foreground color
        canvas.set_draw_color(MOD13_PAL[(cmd.color&0xF) as usize]);
 
        let last_pos = canvas.render_font_string_wrapped(0, -(self.scroll as i32), canvas.window().size().0-if self.last_pos.1 > height { 16 } else { 0 }, &mut self.font_texture, &self.font, cmd.get_text()).unwrap();


        // Render the caret
        if self.is_caret_rendered() {
            canvas.fill_rect(sdl2::rect::Rect::new(self.last_pos.0 as i32, last_pos.1 as i32+(16-6), 8, 3)).unwrap();
        }
        
        self.render_scrollbar(canvas);
    }

    pub fn update(&mut self, wsize: (u32, u32), cmd: &Screen) {
        self.ticks += 1;
        let (width, height) = wsize;

        let last_pos = self.font.get_size_from_string(0, 0, (width as i32)-if self.last_pos.1 > height { 16 } else { 0 }, cmd.get_text());

       
        self.last_pos = (last_pos.0.max(0) as u32, last_pos.1.max(0) as u32);

        if self.scroll_locked {
            self.scroll = self.last_pos.1
        }

        self.scroll = self.scroll.min((self.last_pos.1 as i32-height as i32+16).max(0) as u32);
    }
}

