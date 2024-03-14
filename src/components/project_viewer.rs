use crate::{components::component::Component, Container, ContainerType, Framework};
use crossterm::{
    cursor,
    event::{Event, KeyCode, KeyEvent, KeyEventKind, KeyEventState, KeyModifiers},
    queue,
    style::{self, Color},
};
use std::{
    fs,
    io::Stdout,
    process::exit,
    sync::{Arc, RwLock},
};

pub struct ProjectViewer {
    container: Arc<RwLock<Container>>,
    path: String,
    at_line: usize,

    fs: Filesystem,
}

impl ProjectViewer {
    pub fn new() -> Arc<RwLock<Self>> {
        let mut container = Container::new("ProjectViewer", None);
        container.focus();
        let container = Arc::new(RwLock::new(container));
        let path = std::env::var("PWD").unwrap();
        let res = Arc::new(RwLock::new(ProjectViewer {
            container,
            path: path.clone(),
            at_line: 0,
            fs: Filesystem::new(path),
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
                            if line < (contsize.1 - 2).min(res_ref.read().unwrap().fs.get(contsize.1 - 1).len()-1) {
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
	    let content = self.fs.get(size.1);
            queue!(
                stdout,
                cursor::MoveTo(offset.0 as u16, offset.1 as u16 + 1),
                style::ResetColor
            )
            .unwrap();
            let mut light_line = true;
            for i in 0..(size.1 - 1).min(content.len()) {
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
                    queue!(stdout, style::ResetColor, style::SetBackgroundColor(Color::DarkGrey)).unwrap();
                }
		let line = content[i].clone();
                let line = if line.len() > size.0 {
		    line.split_at(size.0).0.to_string()
		} else {
		    line
		};
		let len = line.len();
		queue!(stdout, style::Print(line)).unwrap();
                for _ in 0..(size.0 - len) {
                    queue!(stdout, style::Print(" ".to_string())).unwrap();
                }
                queue!(stdout, cursor::MoveToColumn(0), cursor::MoveToNextLine(1)).unwrap();
                light_line = !light_line;
            }
        }
    }
}

enum PathType {
    File,
    Directory,
    SymLink,
    None,
}

struct Path(String, PathType, Vec<Box<Self>>);

impl PartialEq for Path {
    fn eq(&self, s: &Self) -> bool {
	self.0 == s.0
    }
}

impl PartialOrd for Path {
    fn partial_cmp(&self, s: &Self) -> Option<Orderibg> {}
}

struct Filesystem {
    root: String,
    path_cache: Vec<Path>,
    showing_start: usize,
}

impl Filesystem {
    pub fn new(root: String) -> Self {
        let mut res = Filesystem {
            root,
            path_cache: Vec::new(),
	    showing_start: 0,
        };
        for entry in fs::read_dir(&res.root).unwrap() {
            let entry = entry.unwrap();
            let ptype = if entry.path().is_file() {
                PathType::File
            } else if entry.path().is_dir() {
                PathType::Directory
            } else if entry.path().is_symlink() {
                PathType::SymLink
            } else {
                PathType::None
            };
            res.path_cache.push(Path(
                entry.file_name().into_string().unwrap(),
                ptype,
                Vec::new(),
            ));
        }
        res
    }

    pub fn get(&self, lines: usize) -> Vec<String> {
	let mut count = 0;
	let mut line = 0;
	let mut res = Vec::new();
	for entry in &self.path_cache {
	    if line >= self.showing_start {
		res.push(entry.0.clone());
		count += 1;
	    }
	    line += 1;
	    if count >= lines {
		break;
	    }
	}
	res
    }
}
