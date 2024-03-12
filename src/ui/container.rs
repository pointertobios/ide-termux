use std::{
    cell::RefCell, io::Stdout, rc::Rc, sync::{Arc, RwLock}
};

use crossterm::{cursor, event::Event, queue, style};

pub enum ContainerType {
    Father {
        subconts: [Option<Arc<RwLock<Container>>>; 2],
        /// 两个子Containero是否是垂直布局
        vert_layout: bool,
        /// 独占
        all_own: bool,
    },
    ProjectViewer,
    Terminal,
    Editor,
    None,
}

pub struct Container {
    name: String,
    x: usize,
    y: usize,
    width: usize,
    height: usize,
    focused: bool,
    cursor_x: usize,
    cursor_y: usize,
    cont_type: ContainerType,

    eve_handler: Option<Rc<RefCell<dyn FnMut(Event)>>>,
}

impl Container {
    pub fn new(name: &String, f: Option<Rc<RefCell<dyn FnMut(Event)>>>) -> Self {
        Container {
            name: name.to_string(),
            x: 0,
            y: 0,
            width: 0,
            height: 0,
            focused: false,
            cursor_x: 0,
            cursor_y: 0,
            cont_type: ContainerType::None,
            eve_handler: f,
        }
    }

    pub fn new_root(width: usize, height: usize, f: Option<Rc<RefCell<dyn FnMut(Event)>>>) -> Self {
        Container {
            name: "RootContainer".to_string(),
            x: 0,
            y: 0,
            width,
            height,
            focused: true,
            cursor_x: 0,
            cursor_y: 0,
            cont_type: ContainerType::Father {
                subconts: [None, None],
                vert_layout: true,
                all_own: false,
            },
            eve_handler: f,
        }
    }

    pub fn get_by_path(&self, path: &[&str]) -> Result<Arc<RwLock<Container>>, String> {
        if path.len() == 1 {
            if let ContainerType::Father { subconts, .. } = &self.cont_type {
                if let Some(down_cont) = &subconts[1] {
                    if down_cont.read().unwrap().name == path[0] {
                        Ok(Arc::clone(down_cont))
                    } else if let Some(up_cont) = &subconts[0] {
                        if up_cont.read().unwrap().name == path[0] {
                            Ok(Arc::clone(up_cont))
                        } else {
                            Err(format!("No container names {}.", path[0]))
                        }
                    } else {
                        Err(format!("No container names {}.", path[0]))
                    }
                } else {
                    Err(format!("No container names {}.", path[0]))
                }
            } else {
                Err(format!("{} is not a father container.", self.name))
            }
        } else {
            if let ContainerType::Father { subconts, .. } = &self.cont_type {
                if let Some(down_cont) = &subconts[1] {
                    if down_cont.read().unwrap().name == path[0] {
                        if let Ok(res) = down_cont.read().unwrap().get_by_path(&path[1..]) {
                            Ok(res)
                        } else {
                            Err(format!("No container names {}.", path[0]))
                        }
                    } else if let Some(up_cont) = &subconts[0] {
                        if up_cont.read().unwrap().name == path[0] {
                            if let Ok(res) = up_cont.read().unwrap().get_by_path(&path[1..]) {
                                Ok(res)
                            } else {
                                Err(format!("No container names {}.", path[0]))
                            }
                        } else {
                            Err(format!("No container names {}.", path[0]))
                        }
                    } else {
                        Err(format!("No container names {}.", path[0]))
                    }
                } else {
                    Err(format!("No container names {}.", path[0]))
                }
            } else {
                Err(format!("{} is not a father container.", self.name))
            }
        }
    }

    pub fn dispatch(&mut self, event: Event) {
        if let Some(handler) = &self.eve_handler {
            handler.borrow_mut()(event);
        }
    }

    pub fn focus(&mut self) {
        self.focused = true;
    }

    pub fn disfocus(&mut self) {
        self.focused = false;
    }

    pub fn focus_path(&mut self, path: &[&str]) {
        if path.len() == 0 {
            return;
        }
        self.focused = true;
        if let ContainerType::Father { subconts, .. } = &self.cont_type {
            if let Some(down_cont) = &subconts[1] {
                if down_cont.read().unwrap().name == path[0] {
                    down_cont.write().unwrap().focus_path(&path[1..]);
                } else {
                    if let Some(up_cont) = &subconts[0] {
                        if up_cont.read().unwrap().name == path[0] {
                            up_cont.write().unwrap().focus_path(&path[1..]);
                        }
                    }
                }
            }
        }
    }

    pub fn disfocus_path(&mut self, path: &[&str]) {
        if path.len() == 0 {
            return;
        }
        self.focused = false;
        if let ContainerType::Father { subconts, .. } = &self.cont_type {
            if let Some(down_cont) = &subconts[1] {
                if down_cont.read().unwrap().name == path[0] {
                    down_cont.write().unwrap().disfocus_path(&path[1..]);
                } else {
                    if let Some(up_cont) = &subconts[0] {
                        if up_cont.read().unwrap().name == path[0] {
                            up_cont.write().unwrap().disfocus_path(&path[1..]);
                        }
                    }
                }
            }
        }
    }

    pub fn is_focused(&self) -> bool {
        self.focused
    }

    pub fn is_father(&self) -> bool {
        if let ContainerType::Father { .. } = &self.cont_type {
            true
        } else {
            false
        }
    }

    pub fn render(&self, father_offset: (usize, usize), stdout: &mut Stdout) {
        if let ContainerType::Father { subconts, .. } = &self.cont_type {
            for cont in subconts {
                if let Some(cont) = cont {
                    cont.read()
                        .unwrap()
                        .render((father_offset.0 + self.x, father_offset.1 + self.y), stdout);
                }
            }
        } else {
            let t = format!(
                "{}: [{}, {}] {}",
                self.name,
                self.width,
                self.height,
                if self.focused { "focused" } else { "unfocused" }
            );
            let t = if t.len() > self.width {
                t.split_at(self.width).0.to_string()
            } else {
                t
            };
            queue!(
                stdout,
                cursor::MoveTo(
                    (father_offset.0 + self.x) as u16,
                    (father_offset.1 + self.y) as u16
                ),
                style::Print(t),
                cursor::MoveTo(
                    (father_offset.0 + self.x + self.width) as u16 - 1,
                    (father_offset.1 + self.y + self.height) as u16 - 1
                ),
                style::Print("+"),
            )
            .unwrap();
        }
    }

    pub fn set_type(&mut self, _type: ContainerType) {
        self.cont_type = _type;
        let (w, h) = (self.width, self.height);
        self.set_size(w, h);
    }

    pub fn set_size(&mut self, width: usize, height: usize) {
        self.width = width;
        self.height = height;
        if let ContainerType::Father {
            subconts,
            vert_layout,
            all_own,
        } = &mut self.cont_type
        {
            if *vert_layout {
                if let Some(up_cont) = &subconts[0] {
                    if let Some(down_cont) = &subconts[1] {
                        let h = if down_cont.read().unwrap().focused {
                            if *all_own {
                                height - 1
                            } else {
                                height / 3 * 2
                            }
                        } else {
                            if *all_own {
                                1
                            } else {
                                height / 3
                            }
                        };
                        down_cont.write().unwrap().set_size(width, h);
                        down_cont.write().unwrap().set_location(0, height - h);
                        up_cont.write().unwrap().set_size(width, height - h);
                    } else {
                        up_cont.write().unwrap().set_size(width, height);
                    }
                } else {
                    if let Some(down_cont) = &subconts[1] {
                        down_cont.write().unwrap().set_size(width, height);
                        down_cont.write().unwrap().set_location(0, 0);
                    }
                }
            } else {
                if let Some(up_cont) = &subconts[0] {
                    if let Some(down_cont) = &subconts[1] {
                        let w = if down_cont.read().unwrap().focused {
                            if *all_own {
                                width - 1
                            } else {
                                width / 3 * 2
                            }
                        } else {
                            if *all_own {
                                1
                            } else {
                                width / 3
                            }
                        };
                        down_cont.write().unwrap().set_size(w, height);
                        down_cont.write().unwrap().set_location(width - w, 0);
                        up_cont.write().unwrap().set_size(width - w, height);
                    } else {
                        up_cont.write().unwrap().set_size(width, height);
                    }
                } else {
                    if let Some(down_cont) = &subconts[1] {
                        down_cont.write().unwrap().set_size(width, height);
                        down_cont.write().unwrap().set_location(0, 0);
                    }
                }
            }
        }
    }

    pub fn set_location(&mut self, x: usize, y: usize) {
        self.x = x;
        self.y = y;
    }

    pub fn add_container(
        &mut self,
        path: &[&str],
        container: Arc<RwLock<Container>>,
    ) -> Result<(), String> {
        if path.len() == 0 {
            if let ContainerType::Father { subconts, .. } = &mut self.cont_type {
                if let Some(_) = &subconts[1] {
                    if let Some(_) = &subconts[0] {
                        Err(format!("{} unable to add a new container.", self.name))?;
                    } else {
                        subconts[0] = Some(container);
                    }
                } else {
                    subconts[1] = Some(container);
                }
            } else {
                Err(format!("{} is not a father container.", self.name))?;
            }
        } else {
            if let ContainerType::Father { subconts, .. } = &mut self.cont_type {
                if let Some(cont) = &subconts[0] {
                    if cont.read().unwrap().name == path[0] {
                        cont.write().unwrap().add_container(&path[1..], container)?;
                    } else if let Some(cont) = &subconts[1] {
                        if cont.read().unwrap().name == path[0] {
                            cont.write().unwrap().add_container(&path[1..], container)?;
                        }
                    } else {
                        Err(format!("No container names {}.", path[0]))?;
                    }
                }
            } else {
                Err(format!("{} is not a father container.", self.name))?;
            }
        }
        Ok(())
    }
}
