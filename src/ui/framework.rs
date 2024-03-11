use std::io::Write;
use crossterm::{
    cursor,
    queue,
    style::Print,
    terminal::{
        enable_raw_mode,
        disable_raw_mode,
        window_size,
	Clear,
	ClearType,
    },
};

pub struct Framework {
    width: usize,
    height: usize,
}

impl Framework {
    pub fn new() -> Self {
        enable_raw_mode().unwrap();
	Framework {
	    width: window_size().unwrap().columns as usize,
	    height: window_size().unwrap().rows as usize,
	}
    }
    pub fn render(&mut self) {
        let mut stdout = std::io::stdout();
	queue!(
	   stdout,
	   Clear(ClearType::All),
	   cursor::MoveTo(0, 0),
	   Print(format!("{},{}", self.width, self.height)),
	).unwrap();
	stdout.flush().unwrap();
    }
    pub fn set_size(&mut self, width: usize, height: usize) {
        self.width = width;
	self.height = height;
    }
}

impl Drop for Framework {
    fn drop(&mut self) {
        disable_raw_mode().unwrap();
    }
}
