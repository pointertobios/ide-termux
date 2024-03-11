use crossterm::{
    cursor, queue,
    terminal::{disable_raw_mode, enable_raw_mode, window_size, Clear, ClearType},
};
use std::{
    io::Write,
    sync::{Arc, RwLock},
};

use super::{container::Container, Event};

pub struct Framework {
    width: usize,
    height: usize,
    container: Option<Arc<RwLock<Container>>>,

    focused_path: String,
}

impl Framework {
    pub fn new() -> Self {
        enable_raw_mode().unwrap();
        queue!(std::io::stdout(), cursor::Hide).unwrap();
        std::io::stdout().flush().unwrap();
        Framework {
            width: window_size().unwrap().columns as usize,
            height: window_size().unwrap().rows as usize,
            container: None,
            focused_path: String::new(),
        }
    }

    pub fn render(&mut self) {
        let mut stdout = std::io::stdout();
        queue!(stdout, Clear(ClearType::All)).unwrap();
        if let Some(container) = &self.container {
            container.read().unwrap().render((0, 0), &mut stdout);
        }
        stdout.flush().unwrap();
    }

    pub fn set_size(&mut self, width: usize, height: usize) {
        self.width = width;
        self.height = height;
        if let Some(container) = &self.container {
            container.write().unwrap().set_size(width, height);
        }
    }

    pub fn get_size(&self) -> (usize, usize) {
        (self.width, self.height)
    }

    pub fn set_container(&mut self, container: Arc<RwLock<Container>>) {
        self.container = Some(container);
    }

    pub fn add_container(
        &mut self,
        path: &str,
        container: Arc<RwLock<Container>>,
    ) -> Result<(), String> {
        if let Some(cont) = &mut self.container {
            let path = path
                .split("/")
                .filter(|&s| !s.is_empty())
                .collect::<Vec<&str>>();
            cont.write()
                .unwrap()
                .add_container(path.as_slice(), container)?;
        }
        Ok(())
    }

    pub fn dispatch(&self, event: Event) {
        match event {
            Event::ChangeFocus(which) => todo!(),
            Event::Crossterm(e) => {
                if let Some(container) = &self.container {
                    container.write().unwrap().dispatch(e);
                }
            }
        }
    }
}

impl Drop for Framework {
    fn drop(&mut self) {
        queue!(std::io::stdout(), cursor::Show).unwrap();
        disable_raw_mode().unwrap();
    }
}
