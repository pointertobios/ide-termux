use crossterm::style::Stylize;
use std::{
    collections::HashMap,
    iter,
    process::exit,
    sync::{Arc, RwLock},
};
use tokio::sync::{mpsc::Receiver, RwLock as AsyncRwLock};

use crate::{
    components::component::Component,
    named_pipe::{NamedPipe, PipeObject},
    renderer::Renderer,
    ui::{
        container::{Container, ContainerType},
        framework::Framework,
    },
};

enum EditorMode {
    Command,
    Edit,
}

pub struct Editor {
    container: Arc<RwLock<Container>>,
    id: usize,
    file: Option<Arc<AsyncRwLock<Editing>>>,
    mode: EditorMode,
    file_open_receiver: Arc<AsyncRwLock<Receiver<PipeObject>>>,
}

impl Editor {
    pub fn new(id: usize) -> Arc<RwLock<Self>> {
        let container = Container::new(&("Editor".to_string() + &id.to_string()), None);
        let container = Arc::new(RwLock::new(container));
        let res = Arc::new(RwLock::new(Editor {
            container,
            id,
            file: None,
            mode: EditorMode::Command,
            file_open_receiver: NamedPipe::open_receiver(format!("FileOpen{}", id)),
        }));
        res.read()
            .unwrap()
            .container
            .write()
            .unwrap()
            .set_type(ContainerType::Editor(Arc::clone(&res)));
        res
    }
}

impl Component for Editor {
    fn bind_to(
        &mut self,
        framework: &mut Framework,
    ) -> Result<(), Box<dyn FnOnce(Framework) -> !>> {
        match framework.add_container("/WorkArea/EditorArea", Arc::clone(&self.container)) {
            Ok(()) => Ok(()),
            Err(s) => Err(Box::new(move |fw| {
                drop(fw);
                println!("{}", s);
                exit(-1)
            })),
        }
    }

    fn render(&mut self, renderer: &Renderer) -> (bool, (usize, usize)) {
        if let Ok(PipeObject::Editing(edi)) = self.file_open_receiver.blocking_write().try_recv() {
            self.file = Some(edi);
        }
        let size = renderer.get_size();
        let focused = self.container.read().unwrap().focused();
        let title = if let Some(f) = &self.file {
            " ".to_string() + &f.blocking_read().path.last().unwrap().clone()
        } else {
            format!(" Editor {}", self.id)
        };
        // 标题
        if !focused {
            let title = title.chars().collect::<Vec<_>>();
            if size.0 == 1 {
                let mut title = if title.len() > size.1 {
                    title.split_at(size.1).0.to_vec()
                } else {
                    title
                };
                if title.len() < size.1 {
                    title.append(
                        &mut iter::repeat(' ')
                            .take(size.1 - title.len())
                            .collect::<Vec<_>>(),
                    );
                }
                for i in 0..title.len() {
                    renderer.set(0, i, title[i].white().on_dark_grey());
                }
            } else {
                let mut title = if title.len() > size.0 {
                    title.split_at(size.1).0.to_vec()
                } else {
                    title
                };
                if title.len() < size.0 {
                    title.append(
                        &mut iter::repeat(' ')
                            .take(size.0 - title.len())
                            .collect::<Vec<_>>(),
                    );
                }
                let title = String::from_iter(title.iter());
                renderer.set_section(0, 0, title.white().on_dark_grey());
            }
        } else {
            let title = title.chars().collect::<Vec<_>>();
            let mut title = if title.len() > size.0 {
                title.split_at(size.1).0.to_vec()
            } else {
                title
            };
            if title.len() < size.0 {
                title.append(
                    &mut iter::repeat(' ')
                        .take(size.0 - title.len())
                        .collect::<Vec<_>>(),
                );
            }
            let title = String::from_iter(title.iter());
            renderer.set_section(0, 0, title.dark_red().on_dark_blue());
        }
        // 内容
        if size.0 > 1 {
            let mut linen = 1;
            while linen < size.1 {
                let l = iter::repeat(' ').take(size.0).collect::<Vec<_>>();
                let l = String::from_iter(&mut l.iter());
                renderer.set_section(0, linen, l.reset());
                linen += 1;
            }
        }
        (false, (0, 0))
    }
}

pub struct Editing {
    path: Vec<String>,
    buffer: HashMap<usize, LineDiff>,
    showing_start: usize,
    showing_length: usize,
}

impl Editing {
    pub fn new(path: Vec<String>) -> Self {
        Editing {
            path,
            buffer: HashMap::new(),
            showing_start: 0,
            showing_length: 0,
        }
    }
}

struct LineDiff {
    origin_offset: usize,
    /// 对于一个文件中原有的行，这个vec总是以换行结尾，即使是空行
    /// 当这个vec长度为0时，说明是新加入的行
    origin_content: Vec<char>,
    inserts: (),
}
