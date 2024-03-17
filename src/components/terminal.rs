use std::{
    iter,
    process::exit,
    sync::{Arc, RwLock},
};

use crossterm::style::{Color, Stylize};

use crate::{
    components::component::Component,
    renderer::Renderer,
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

    fn render(&self, renderer: &Renderer) -> (bool, (usize, usize)) {
        let size = renderer.get_size();
        let title = format!("Terminal {}", self.shell);
        let title = title.chars().collect::<Vec<_>>();
        let mut title = if title.len() > size.0 {
            title.split_at(size.0).0.to_vec()
        } else {
            title
        };
        title.append(
            &mut iter::repeat(' ')
                .take(size.0 - title.len())
                .collect::<Vec<_>>(),
        );
        let title = String::from_iter(title.iter());
        if !self.container.read().unwrap().focused() {
            renderer.set_section(0, 0, title.white().on_dark_grey());
        } else {
            // 绘制标题
            renderer.set_section(0, 0, title.dark_red().on_dark_blue());
            // 绘制主体
            let mut linen = 1;
            // 覆盖不需要的
            while linen < size.1 {
                renderer.set_section(
                    0,
                    linen,
                    iter::repeat(' ').take(size.0).collect::<String>().reset(),
                );
                linen += 1;
            }
        }
        (false, (0, 0))
    }
}
