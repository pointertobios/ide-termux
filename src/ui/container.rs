use crate::{
    components::{
        component::Component, editor::Editor, project_viewer::ProjectViewer, terminal::Terminal,
    },
    renderer::Renderer,
};
use crossterm::event::Event;
use std::sync::{Arc, RwLock};

pub enum ContainerType {
    Father {
        subconts: [Option<Arc<RwLock<Container>>>; 2],
        /// 两个子Containero是否是垂直布局
        vert_layout: bool,
        /// 独占
        all_own: bool,
    },
    ProjectViewer(Arc<RwLock<ProjectViewer>>),
    Terminal(Arc<RwLock<Terminal>>),
    Editor(Arc<RwLock<Editor>>),
    None,
}

/// 这是一个闭包，闭包中不可以对带锁的Container对象解锁
type EventHandler = dyn FnMut(Event, (usize, usize));

pub struct Container {
    name: String,
    x: usize,
    y: usize,
    width: usize,
    height: usize,
    focused: bool,
    cont_type: ContainerType,

    eve_handler: Option<Box<EventHandler>>,
}

impl Container {
    pub fn new(name: &str, f: Option<Box<EventHandler>>) -> Self {
        Container {
            name: name.to_string(),
            x: 0,
            y: 0,
            width: 0,
            height: 0,
            focused: false,
            cont_type: ContainerType::None,
            eve_handler: f,
        }
    }

    pub fn new_root(width: usize, height: usize, f: Option<Box<EventHandler>>) -> Self {
        Container {
            name: "RootContainer".to_string(),
            x: 0,
            y: 0,
            width,
            height,
            focused: true,
            cont_type: ContainerType::Father {
                subconts: [None, None],
                vert_layout: true,
                all_own: true,
            },
            eve_handler: f,
        }
    }

    pub fn focused(&self) -> bool {
        self.focused
    }

    pub fn get_size(&self) -> (usize, usize) {
        (self.width, self.height)
    }

    pub fn get_location(&self) -> (usize, usize) {
        (self.x, self.y)
    }

    // 以后有可能用得上
    // pub fn get_by_path(&self, path: &[&str]) -> Result<Arc<RwLock<Container>>, String> {
    //     if path.len() == 1 {
    //         if let ContainerType::Father { subconts, .. } = &self.cont_type {
    //             if let Some(down_cont) = &subconts[1] {
    //                 if down_cont.read().unwrap().name == path[0] {
    //                     Ok(Arc::clone(down_cont))
    //                 } else if let Some(up_cont) = &subconts[0] {
    //                     if up_cont.read().unwrap().name == path[0] {
    //                         Ok(Arc::clone(up_cont))
    //                     } else {
    //                         Err(format!("No container names {}.", path[0]))
    //                     }
    //                 } else {
    //                     Err(format!("No container names {}.", path[0]))
    //                 }
    //             } else {
    //                 Err(format!("No container names {}.", path[0]))
    //             }
    //         } else {
    //             Err(format!("{} is not a father container.", self.name))
    //         }
    //     } else {
    //         if let ContainerType::Father { subconts, .. } = &self.cont_type {
    //             if let Some(down_cont) = &subconts[1] {
    //                 if down_cont.read().unwrap().name == path[0] {
    //                     if let Ok(res) = down_cont.read().unwrap().get_by_path(&path[1..]) {
    //                         Ok(res)
    //                     } else {
    //                         Err(format!("No container names {}.", path[0]))
    //                     }
    //                 } else if let Some(up_cont) = &subconts[0] {
    //                     if up_cont.read().unwrap().name == path[0] {
    //                         if let Ok(res) = up_cont.read().unwrap().get_by_path(&path[1..]) {
    //                             Ok(res)
    //                         } else {
    //                             Err(format!("No container names {}.", path[0]))
    //                         }
    //                     } else {
    //                         Err(format!("No container names {}.", path[0]))
    //                     }
    //                 } else {
    //                     Err(format!("No container names {}.", path[0]))
    //                 }
    //             } else {
    //                 Err(format!("No container names {}.", path[0]))
    //             }
    //         } else {
    //             Err(format!("{} is not a father container.", self.name))
    //         }
    //     }
    // }

    pub fn set_handler(&mut self, f: Box<EventHandler>) {
        self.eve_handler = Some(f);
    }

    pub fn dispatch(&mut self, event: Event) {
        let size = self.get_size();
        if let ContainerType::Father { subconts, .. } = &mut self.cont_type {
            for cont in subconts {
                if let Some(cont) = cont {
                    cont.write().unwrap().dispatch(event.clone());
                }
            }
        } else if self.focused {
            if let Some(handler) = &mut self.eve_handler {
                (*handler)(event, size);
            }
        }
    }

    pub fn focus(&mut self) {
        self.focused = true;
    }

    pub fn focus_path(&mut self, path: &[&str]) {
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

    pub fn render(&self, renderer: &Renderer) -> (bool, (usize, usize)) {
        match &self.cont_type {
            ContainerType::Father { subconts, .. } => {
                let mut res = None;
                for cont in subconts {
                    if let Some(cont) = cont {
                        let size = cont.read().unwrap().get_size();
                        let location = cont.read().unwrap().get_location();
                        let subrend =
                            Renderer::new(renderer.x + location.0, renderer.y + location.1, size.0, size.1);
                        let r = cont.read().unwrap().render(&subrend);
                        if cont.read().unwrap().focused {
                            res = Some(r);
                        }
                    }
                }
                if let Some(res) = res {
                    res
                } else {
                    (false, (0, 0))
                }
            }
            ContainerType::ProjectViewer(proj_viewer) => {
                proj_viewer.write().unwrap().render(renderer)
            }
            ContainerType::Terminal(terminal) => terminal.write().unwrap().render(renderer),
            ContainerType::Editor(editor) => editor.write().unwrap().render(renderer),
            _ => (false, (0, 0)),
        }
    }

    pub fn set_type(&mut self, _type: ContainerType) {
        self.cont_type = _type;
        let (w, h) = (self.width, self.height);
        self.set_size(w, h);
    }

    pub fn set_size(&mut self, width: usize, height: usize) {
        if let Some(f) = &mut self.eve_handler {
            f(Event::Resize(width as u16, height as u16), (width, height));
        }
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
                            if !up_cont.read().unwrap().focused {
                                height / 2
                            } else if *all_own {
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
