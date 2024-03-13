use std::{sync::{Arc, RwLock}, process::exit};
use crate::{Container, ContainerType, Framework};

pub struct ProjectViewer {
    container: Arc<RwLock<Container>>,
}

impl ProjectViewer {
    pub fn new() -> Self {
        let mut container = Container::new("ProjectViewer", None);
	container.set_type(ContainerType::ProjectViewer);
	let container = Arc::new(RwLock::new(container));
        ProjectViewer { container }
    }

    pub fn bind_to(&self, framework: &mut Framework) -> Result<(), Box<dyn FnOnce(Framework)->!>> {
        match framework.add_container("/WorkArea", Arc::clone(&self.container)) {
	    Ok(()) => Ok(()),
	    Err(s) => Err(Box::new(move |fw| {
	            drop(fw);
		    println!("{}", s);
		    exit(-1)
	        }))
        }
    }
}
