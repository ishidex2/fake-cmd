#[derive(PartialEq, Eq, Debug)]
pub enum FontArrangment {
    ASCII
}

#[derive(Debug)]
pub struct Font {
    pub arrangment: FontArrangment,
    pub glyph_size: (usize, usize),
    pub sheet_width: usize
}

impl Font {
    // I'm not sure if there's a better way to do this, so this
    // is an iterative solution that is quite expensive
    // but will do it for us
    pub fn get_size_from_string(&self, x: i32, y: i32, right_bound: i32, string: &str) -> (i32, i32) {
        let mut mutx = x;
        let mut muty = y;

        for chr in string.chars() {
            match chr {
                '\n' => {
                    muty += self.glyph_size.1 as i32;
                    mutx = x;
                },
                '\t' => {
                    mutx += self.glyph_size.0 as i32 * 4;
                },
                _ => {
                    mutx += self.glyph_size.0 as i32;
                }
            }
            if (mutx+self.glyph_size.0 as i32) > (right_bound as i32) {
                muty += self.glyph_size.1 as i32;
                mutx = x;
            }
        }
        (mutx, muty)
    }

    pub fn get_offset(&self, offset: usize) -> (usize, usize) {
        return ((offset*self.glyph_size.0) % (self.sheet_width+1),
        ((offset*self.glyph_size.0) / self.sheet_width)*self.glyph_size.1);
    }

    pub fn get_glyph_rect(&self, chr: char) -> (usize, usize, usize, usize) {
        match self.arrangment {
            FontArrangment::ASCII => {
                let char_id = crate::cp437::unicode_to_cp437(chr) as usize;
                let a = self.get_offset(char_id);
                let b = self.glyph_size;
                return (a.0, a.1, b.0, b.1);
            }
        }
    }

    pub fn get_glyph_rect_sdl(&self, chr: char) -> sdl2::rect::Rect {
        let rect = self.get_glyph_rect(chr);
        sdl2::rect::Rect::new(rect.0 as i32, rect.1 as i32, rect.2 as u32, rect.3 as u32)
    }
}
