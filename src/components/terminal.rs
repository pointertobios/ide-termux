use std::{
    process::exit,
    sync::{Arc, RwLock},
};

use crossterm::style::{Color, Stylize};

use crate::{
    components::component::Component,
    pseudo_mt::PseudoMultithreading,
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

    fn render(&self, renderer: Arc<RwLock<Renderer>>) -> (bool, (usize, usize)) {
        let size = renderer.read().unwrap().get_size();
        let title = format!("Terminal {}", self.shell);
        let title = title.chars().collect::<Vec<_>>();
        let title = if title.len() > size.0 {
            title.split_at(size.0).0.to_vec()
        } else {
            title
        };
        if !self.container.read().unwrap().focused() {
            for i in 0..size.0 {
                if i < title.len() {
                    renderer
                        .read()
                        .unwrap()
                        .set(i, 0, title[i].with(Color::White).on_dark_grey());
                } else {
                    renderer
                        .read()
                        .unwrap()
                        .set(i, 0, ' '.with(Color::White).on_dark_grey());
                }
            }
        } else {
            // 绘制标题
            for i in 0..size.0 {
                if i < title.len() {
                    renderer.read().unwrap().set(
                        i,
                        0,
                        title[i].with(Color::DarkRed).on_dark_blue(),
                    );
                } else {
                    renderer
                        .read()
                        .unwrap()
                        .set(i, 0, ' '.with(Color::DarkRed).on_dark_blue());
                }
            }
            // 绘制主体
            let mut linen = 1;
            // 覆盖不需要的
            let mut pmt = PseudoMultithreading::new();
            while linen < size.1 {
                for i in 0..size.0 {
                    let rd = Arc::clone(&renderer);
                    pmt.add(Box::new(move || {
                        rd.read().unwrap().set(i, linen, ' '.reset());
                    }));
                }
                linen += 1;
            }
            pmt.run();
        }
        (false, (0, 0))
    }
}
