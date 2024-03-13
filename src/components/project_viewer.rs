use crate::{components::component::Component, Container, ContainerType, Framework};
use std::{
    process::exit,
    sync::{Arc, RwLock},
};

pub struct ProjectViewer {
    container: Arc<RwLock<Container>>,
    path: Option<String>,
}

impl ProjectViewer {
    pub fn new() -> Self {
        let mut container = Container::new("ProjectViewer", None);
        container.set_type(ContainerType::ProjectViewer);
        let container = Arc::new(RwLock::new(container));
        ProjectViewer {
            container,
            path: None,
        }
    }
}

impl Component for ProjectViewer {
    fn bind_to(
        &mut self,
        framework: &mut Framework,
    ) -> Result<(), Box<dyn FnOnce(Framework) -> !>> {
        self.path = Some("/WorkArea/ProjectViewer".to_string());
        match framework.add_container("/WorkArea", Arc::clone(&self.container)) {
            Ok(()) => Ok(()),
            Err(s) => Err(Box::new(move |fw| {
                drop(fw);
                println!("{}", s);
                exit(-1)
            })),
        }
    }
}
