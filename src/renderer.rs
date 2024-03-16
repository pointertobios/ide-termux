use crossterm::{
    cursor, queue,
    style::{self, StyledContent},
};

pub struct Renderer {
    x: usize,
    y: usize,
    width: usize,
    height: usize,
}

impl Renderer {
    pub fn new(x: usize, y: usize, width: usize, height: usize) -> Self {
        Renderer {
            x,
            y,
            width,
            height,
        }
    }

    pub fn get_size(&self) -> (usize, usize) {
        (self.width, self.height)
    }

    pub fn set(&self, x: usize, y: usize, ch: StyledContent<char>) {
        if x > self.width || y > self.height {
            return;
        }
        queue!(
            std::io::stdout(),
            cursor::MoveTo((self.x + x) as u16, (self.y + y) as u16),
            style::PrintStyledContent(ch)
        )
        .unwrap();
    }

    pub fn set_section(&self, x: usize, y: usize, st: StyledContent<String>) {
        if x > self.width || y > self.height {
            return;
        }
        queue!(
            std::io::stdout(),
            cursor::MoveTo((self.x + x) as u16, (self.y + y) as u16),
            style::PrintStyledContent(st)
        )
        .unwrap();
    }
}
