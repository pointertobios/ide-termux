use crossterm::style::{Color, Stylize};

use crate::{
    renderer::Renderer,
    ui::{
        container::{Container, ContainerType},
        framework::Framework,
    },
};
use std::{
    collections::HashMap,
    iter,
    process::exit,
    sync::{Arc, RwLock},
};

use super::component::Component;

enum EditorMode {
    Command,
    Edit,
}

pub struct Editor {
    container: Arc<RwLock<Container>>,
    id: usize,
    file: Option<Arc<RwLock<Editing>>>,
    mode: EditorMode,
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

    fn render(&self, renderer: &Renderer) -> (bool, (usize, usize)) {
        let size = renderer.get_size();
        let focused = self.container.read().unwrap().focused();
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
    pub fn new(path: Vec<String>, showing_start: usize, showing_length: usize) -> Self {
        Editing {
            path,
            buffer: HashMap::new(),
            showing_start,
            showing_length,
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
