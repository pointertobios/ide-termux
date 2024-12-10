use crossterm::{
    cursor::{self, MoveTo},
    queue,
    terminal::{
        disable_raw_mode, enable_raw_mode, window_size, EnterAlternateScreen, LeaveAlternateScreen,
    },
};
use std::{
    collections::HashMap,
    io::Write,
    sync::{Arc, RwLock},
};

use crate::renderer::Renderer;

use super::{container::Container, ChangeFocusEvent, Event};

pub struct Framework {
    width: usize,
    height: usize,
    container: Option<Arc<RwLock<Container>>>,

    focused_path: String,

    path_ajac_table: HashMap<
        String,
        (
            Option<String>,
            Option<String>,
            Option<String>,
            Option<String>,
        ),
    >,
}

impl Framework {
    pub fn new() -> Self {
        enable_raw_mode().unwrap();
        queue!(
            std::io::stdout(),    //
            EnterAlternateScreen, //
            cursor::Hide
        )
        .unwrap();
        std::io::stdout().flush().unwrap();
        let mut framework = Framework {
            width: window_size().unwrap().columns as usize,
            height: window_size().unwrap().rows as usize,
            container: None,
            focused_path: String::new(),
            path_ajac_table: HashMap::new(),
        };
        let root = Container::new_root(framework.get_size().0, framework.get_size().1, None);
        let root = Arc::new(RwLock::new(root));
        framework.set_container(Arc::clone(&root));
        framework.set_focused_path("/WorkArea/ProjectViewer");
        framework
    }

    pub fn render(&mut self) {
        let mut stdout = std::io::stdout();
        queue!(stdout, cursor::Hide).unwrap();
        if let Some(container) = &self.container {
            let renderer = Renderer::new(0, 0, self.width, self.height);
            let location = container.read().unwrap().render(&renderer);
            if location.0 {
                queue!(
                    stdout,
                    MoveTo(location.1 .0 as u16, location.1 .1 as u16),
                    cursor::Show,
                )
                .unwrap();
            } else {
                queue!(stdout, cursor::Hide).unwrap();
            }
        }
        stdout.flush().unwrap();
    }

    pub fn set_focused_path(&mut self, path: &str) {
        self.focused_path = path.to_string();
    }

    pub fn set_size(&mut self, width: usize, height: usize) {
        let width = if width < 11 { 11 } else { width };
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

    pub fn set_adjacy(
        &mut self,
        key: String,
        val: (
            Option<String>,
            Option<String>,
            Option<String>,
            Option<String>,
        ),
    ) {
        self.path_ajac_table.insert(key, val);
    }

    pub fn dispatch(&mut self, event: Event) {
        match event {
            Event::ChangeFocus(which) => {
                match which {
                    ChangeFocusEvent::Up => {
                        if let Some(container) = &self.container {
                            let bpath = self
                                .focused_path
                                .split("/")
                                .filter(|x| !x.is_empty())
                                .collect::<Vec<&str>>();
                            let new_path =
                                if let Some(path) = self.path_ajac_table.get(&self.focused_path) {
                                    let rpath = &path.0;
                                    if let Some(rrpath) = rpath {
                                        let rpath = rrpath
                                            .split("/")
                                            .filter(|x| !x.is_empty())
                                            .collect::<Vec<&str>>();
                                        container.write().unwrap().disfocus_path(&bpath);
                                        container.write().unwrap().focus_path(&rpath);
                                        rrpath.to_string()
                                    } else {
                                        self.focused_path.clone()
                                    }
                                } else {
                                    self.focused_path.clone()
                                };
                            self.focused_path = new_path;
                        }
                    }
                    ChangeFocusEvent::Down => {
                        if let Some(container) = &self.container {
                            let bpath = self
                                .focused_path
                                .split("/")
                                .filter(|x| !x.is_empty())
                                .collect::<Vec<&str>>();
                            let new_path =
                                if let Some(path) = self.path_ajac_table.get(&self.focused_path) {
                                    let rpath = &path.1;
                                    if let Some(rrpath) = rpath {
                                        let rpath = rrpath
                                            .split("/")
                                            .filter(|x| !x.is_empty())
                                            .collect::<Vec<&str>>();
                                        container.write().unwrap().disfocus_path(&bpath);
                                        container.write().unwrap().focus_path(&rpath);
                                        rrpath.to_string()
                                    } else {
                                        self.focused_path.clone()
                                    }
                                } else {
                                    self.focused_path.clone()
                                };
                            self.focused_path = new_path;
                        }
                    }
                    ChangeFocusEvent::Left => {
                        if let Some(container) = &self.container {
                            let bpath = self
                                .focused_path
                                .split("/")
                                .filter(|x| !x.is_empty())
                                .collect::<Vec<&str>>();
                            let new_path =
                                if let Some(path) = self.path_ajac_table.get(&self.focused_path) {
                                    let rpath = &path.2;
                                    if let Some(rrpath) = rpath {
                                        let rpath = rrpath
                                            .split("/")
                                            .filter(|x| !x.is_empty())
                                            .collect::<Vec<&str>>();
                                        container.write().unwrap().disfocus_path(&bpath);
                                        container.write().unwrap().focus_path(&rpath);
                                        rrpath.to_string()
                                    } else {
                                        self.focused_path.clone()
                                    }
                                } else {
                                    self.focused_path.clone()
                                };
                            self.focused_path = new_path;
                        }
                    }
                    ChangeFocusEvent::Right => {
                        if let Some(container) = &self.container {
                            let bpath = self
                                .focused_path
                                .split("/")
                                .filter(|x| !x.is_empty())
                                .collect::<Vec<&str>>();
                            let new_path =
                                if let Some(path) = self.path_ajac_table.get(&self.focused_path) {
                                    let rpath = &path.3;
                                    if let Some(rrpath) = rpath {
                                        let rpath = rrpath
                                            .split("/")
                                            .filter(|x| !x.is_empty())
                                            .collect::<Vec<&str>>();
                                        container.write().unwrap().disfocus_path(&bpath);
                                        container.write().unwrap().focus_path(&rpath);
                                        rrpath.to_string()
                                    } else {
                                        self.focused_path.clone()
                                    }
                                } else {
                                    self.focused_path.clone()
                                };
                            self.focused_path = new_path;
                        }
                    }
                }
                if let Some(container) = &self.container {
                    container.write().unwrap().set_size(self.width, self.height);
                }
            }
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
        queue!(std::io::stdout(), cursor::Show, LeaveAlternateScreen).unwrap();
        disable_raw_mode().unwrap();
    }
}
