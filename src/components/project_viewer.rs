use crate::{components::component::Component, Container, ContainerType, Framework};
use std::{
    io::Stdout,
    process::exit,
    sync::{Arc, RwLock},
};
use crossterm::{queue, style::{self, Color}, cursor};

pub struct ProjectViewer {
    container: Arc<RwLock<Container>>,
    path: String,
}

impl ProjectViewer {
    pub fn new() -> Arc<RwLock<Self>> {
        let mut container = Container::new("ProjectViewer", None);
        container.focus();
        let container = Arc::new(RwLock::new(container));
        let res = Arc::new(RwLock::new(ProjectViewer {
            container,
            path: std::env::var("PWD").unwrap(),
        }));
        res.read()
            .unwrap()
            .container
            .write()
            .unwrap()
            .set_type(ContainerType::ProjectViewer(Arc::clone(&res)));
        res
    }

    pub fn render(&self, offset: (usize, usize), size: (usize, usize), stdout: &mut Stdout) {
        // 颜色方案
        if size.0 == 1 {
	    queue!(stdout, style::SetBackgroundColor(Color::DarkGrey), style::SetForegroundColor(Color::White)).unwrap();
	    let tt = format!("ProjViewer {}", self.path.split("/").collect::<Vec<&str>>().last().unwrap());
	    let tt = if tt.len() > size.1 {
	        tt.split_at(size.1).0.to_string()
	    } else {
	        tt
	    };
	    queue!(stdout, cursor::MoveTo(offset.0 as u16, offset.1 as u16)).unwrap();
	    for c in tt.chars() {
	        queue!(stdout, style::Print(c), cursor::MoveToNextLine(1)).unwrap();
	    }
	    for _ in 0..(size.1 - tt.len()) {
	        queue!(stdout, style::Print(" ".to_string()), cursor::MoveToNextLine(1)).unwrap();
	    }
	}
    }
}

impl Component for ProjectViewer {
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
}
