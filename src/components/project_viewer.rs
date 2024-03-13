use crate::{components::component::Component, Container, ContainerType, Framework};
use crossterm::{
    cursor,
    event::{Event, KeyCode, KeyEvent, KeyEventKind, KeyEventState, KeyModifiers},
    queue,
    style::{self, Color},
};
use std::{
    io::Stdout,
    process::exit,
    sync::{Arc, RwLock},
};

pub struct ProjectViewer {
    container: Arc<RwLock<Container>>,
    path: String,
    at_line: usize,
}

impl ProjectViewer {
    pub fn new() -> Arc<RwLock<Self>> {
        let mut container = Container::new("ProjectViewer", None);
        container.focus();
        let container = Arc::new(RwLock::new(container));
        let res = Arc::new(RwLock::new(ProjectViewer {
            container,
            path: std::env::var("PWD").unwrap(),
            at_line: 0,
        }));
        res.read()
            .unwrap()
            .container
            .write()
            .unwrap()
            .set_type(ContainerType::ProjectViewer(Arc::clone(&res)));
        let res_ref = Arc::clone(&res);
        res.read()
            .unwrap()
            .container
            .write()
            .unwrap()
            .set_handler(Box::new(move |event, contsize| {
                let line = res_ref.read().unwrap().at_line;
                match event {
                    Event::Key(KeyEvent {
                        code,
                        modifiers: KeyModifiers::NONE,
                        kind: KeyEventKind::Press | KeyEventKind::Repeat,
                        state: KeyEventState::NONE,
                    }) => match code {
                        KeyCode::Up => {
                            if line > 0 {
                                res_ref.write().unwrap().at_line -= 1;
                            }
                        }
                        KeyCode::Down => {
                            if line < contsize.1 - 2 {
                                res_ref.write().unwrap().at_line += 1;
                            }
                        }
                        _ => (),
                    },
                    _ => (),
                }
            }));
        res
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

    fn render(&self, offset: (usize, usize), size: (usize, usize), stdout: &mut Stdout) {
        queue!(stdout, cursor::MoveTo(offset.0 as u16, offset.1 as u16)).unwrap();
        if size.0 == 1 {
            queue!(
                stdout,
                style::SetBackgroundColor(Color::DarkGrey),
                style::SetForegroundColor(Color::White)
            )
            .unwrap();
            let tt = format!(
                "ProjViewer {}",
                self.path.split("/").collect::<Vec<&str>>().last().unwrap()
            );
            let tt = if tt.len() > size.1 {
                tt.split_at(size.1).0.to_string()
            } else {
                tt
            };
            for c in tt.chars() {
                queue!(stdout, style::Print(c), cursor::MoveToNextLine(1)).unwrap();
            }
            for _ in 0..(size.1 - tt.len()) {
                queue!(
                    stdout,
                    style::Print(" ".to_string()),
                    cursor::MoveToNextLine(1)
                )
                .unwrap();
            }
        } else {
            // 标题
            queue!(
                stdout,
                style::SetBackgroundColor(Color::DarkBlue),
                style::SetForegroundColor(Color::Red)
            )
            .unwrap();
            let tt = self
                .path
                .split("/")
                .collect::<Vec<&str>>()
                .last()
                .unwrap()
                .to_string();
            let tt = if tt.len() > size.0 {
                tt.split_at(size.1).0.to_string()
            } else {
                tt
            };
            queue!(stdout, style::Print(tt.clone())).unwrap();
            for _ in 0..(size.0 - tt.len()) {
                queue!(stdout, style::Print(" ".to_string()),).unwrap();
            }
            // 主体
            queue!(
                stdout,
                cursor::MoveTo(offset.0 as u16, offset.1 as u16 + 1),
                style::ResetColor
            )
            .unwrap();
            let mut light_line = true;
            for i in 0..(size.1 - 1) {
                if i == self.at_line {
                    queue!(
                        stdout,
                        style::SetBackgroundColor(Color::Grey),
                        style::SetForegroundColor(Color::Black)
                    )
                    .unwrap();
                } else if light_line {
                    queue!(stdout, style::ResetColor).unwrap();
                } else {
                    queue!(stdout, style::SetBackgroundColor(Color::DarkGrey)).unwrap();
                }
                let len = 0;
                for _ in 0..(size.0 - len) {
                    queue!(stdout, style::Print(" ".to_string())).unwrap();
                }
                queue!(stdout, cursor::MoveToColumn(0), cursor::MoveToNextLine(1)).unwrap();
                light_line = !light_line;
            }
        }
    }
}
