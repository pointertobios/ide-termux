use std::{
    process::exit,
    sync::{Arc, RwLock},
};

use crate::{
    renderer::Renderer,
    ui::{
        container::{Container, ContainerType},
        framework::Framework,
    },
};

use super::component::Component;

pub struct EditorArea {
    container: Arc<RwLock<Container>>,
}

impl EditorArea {
    pub fn new() -> Arc<RwLock<Self>> {
        let mut container = Container::new("EditorArea", None);
        container.set_type(ContainerType::Father {
            subconts: [None, None],
            vert_layout: true,
            all_own: true,
        });
        let container = Arc::new(RwLock::new(container));
        Arc::new(RwLock::new(EditorArea { container }))
    }
}

impl Component for EditorArea {
    fn bind_to(
        &mut self,
        framework: &mut Framework,
    ) -> Result<(), Box<dyn FnOnce(Framework) -> !>> {
        match framework.add_container("/WorkArea", Arc::clone(&self.container)) {
            Ok(()) => Ok(()),
            Err(s) => Err(Box::new(move |fw| {
                drop(fw);
                println!("{}", s);
                exit(-1)
            })),
        }
    }

    fn render(&self, _renderer: &Renderer) -> (bool, (usize, usize)) {
        (false, (0, 0))
    }
}

pub struct WorkArea {
    container: Arc<RwLock<Container>>,
}

impl WorkArea {
    pub fn new() -> Arc<RwLock<Self>> {
        let mut container = Container::new("WorkArea", None);
        container.focus();
        container.set_type(ContainerType::Father {
            subconts: [None, None],
            vert_layout: false,
            all_own: true,
        });
        let container = Arc::new(RwLock::new(container));
        Arc::new(RwLock::new(WorkArea { container }))
    }
}

impl Component for WorkArea {
    fn bind_to(
        &mut self,
        framework: &mut Framework,
    ) -> Result<(), Box<dyn FnOnce(Framework) -> !>> {
        match framework.add_container("/", Arc::clone(&self.container)) {
            Ok(()) => Ok(()),
            Err(s) => Err(Box::new(move |fw| {
                drop(fw);
                println!("{}", s);
                exit(-1)
            })),
        }
    }

    fn render(&self, _renderer: &Renderer) -> (bool, (usize, usize)) {
        (false, (0, 0))
    }
}
