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

pub struct Editor {
    container: Arc<RwLock<Container>>,
    id: usize,
}

impl Editor {
    pub fn new(id: usize) -> Arc<RwLock<Self>> {
        let container = Container::new(&("Editor".to_string() + &id.to_string()), None);
        let container = Arc::new(RwLock::new(container));
        let res = Arc::new(RwLock::new(Editor { container, id }));
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

    fn render(&self, renderer: &mut Renderer) -> (bool, (usize, usize)) {
        (false, (0, 0))
    }
}
