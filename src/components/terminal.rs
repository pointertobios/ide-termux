use std::{
    process::exit,
    sync::{Arc, RwLock},
};

use crossterm::{
    cursor, queue,
    style::{self, Color},
};

use crate::{
    components::component::Component,
    ui::{
        container::{Container, ContainerType},
        framework::Framework,
    },
};

pub struct Terminal {
    container: Arc<RwLock<Container>>,
    shell: String,
}

impl Terminal {
    pub fn new() -> Arc<RwLock<Self>> {
        let container = Container::new("Terminal", None);
        let container = Arc::new(RwLock::new(container));
        let res = Arc::new(RwLock::new(Self {
            container,
            shell: std::env::var("SHELL").unwrap(),
        }));
        res.read()
            .unwrap()
            .container
            .write()
            .unwrap()
            .set_type(ContainerType::Terminal(Arc::clone(&res)));
        let res_ref = Arc::clone(&res);
        res.read()
            .unwrap()
            .container
            .write()
            .unwrap()
            .set_handler(Box::new(move |event, contsize| {}));
        res
    }
}

impl Component for Terminal {
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

    fn render(&self, offset: (usize, usize), size: (usize, usize), stdout: &mut std::io::Stdout) {
        queue!(
            stdout,
            style::ResetColor,
            cursor::MoveTo(offset.0 as u16, offset.1 as u16),
            style::SetBackgroundColor(Color::DarkBlue),
            style::SetForegroundColor(Color::Red),
        )
        .unwrap();
        let tt = format!("Terminal {}", self.shell);
        let tt = if tt.len() > size.0 {
            tt.split_at(size.1).0.to_string()
        } else {
            tt
        };
        queue!(stdout, style::Print(tt.clone())).unwrap();
        for _ in 0..(size.0 - tt.len()) {
            queue!(stdout, style::Print(" ".to_string()),).unwrap();
        }
        queue!(stdout, style::ResetColor).unwrap();
    }
}
