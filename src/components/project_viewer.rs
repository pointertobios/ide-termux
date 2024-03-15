use crate::{components::component::Component, Container, ContainerType, Framework};
use crossterm::{
    cursor,
    event::{Event, KeyCode, KeyEvent, KeyEventKind, KeyEventState, KeyModifiers},
    queue,
    style::{self, Color},
};
use std::{
    cell::RefCell,
    cmp::Ordering,
    fs,
    io::Stdout,
    process::exit,
    rc::Rc,
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
                            if line
                                < (contsize.1 - 2).min(
                                    res_ref
                                        .read()
                                        .unwrap()
                                        .fs
                                        .iter(contsize.1 - 1)
                                        .collect::<Vec<_>>()
                                        .len()
                                        - 1,
                                )
                            {
                                res_ref.write().unwrap().at_line += 1;
                            }
                        }
                        KeyCode::Enter => {
                            let content = res_ref
                                .read()
                                .unwrap()
                                .fs
                                .iter(contsize.1 - 1)
                                .collect::<Vec<_>>();
                            let meta = &content[res_ref.read().unwrap().at_line];
                            if meta.1 == PathType::Directory {
                                res_ref.write().unwrap().fs.fold_unfold(&meta.0, None);
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
            let mut i = 0;
            for line_meta in self.fs.iter(size.1 - 1) {
                if i == self.at_line {
                    queue!(
                        stdout,
                        style::SetBackgroundColor(Color::Grey),
                        style::SetForegroundColor(Color::Black)
                    )
                    .unwrap();
                } else {
                    queue!(stdout, style::ResetColor).unwrap();
                }
                let line = {
                    let mut s = String::new();
                    if line_meta.1 == PathType::Directory {
                        for i in 0..line_meta.3 {
                            if i + 1 == line_meta.3 {
                                if *line_meta.4.last().unwrap() {
                                    s += "┕━"
                                } else {
                                    s += "┝━";
                                }
                            } else {
                                s += if line_meta.4[i + 1] { "  " } else { "│ " };
                            }
                        }
                        if line_meta.2 {
                            s += "┭ ";
                        } else {
                            s += "╾ ";
                        }
                    } else {
                        for i in 0..line_meta.3 {
                            if i + 1 == line_meta.3 {
                                if *line_meta.4.last().unwrap() {
                                    s += "╰─";
                                } else {
                                    s += "├─";
                                }
                            } else {
                                s += if line_meta.4[i + 1] { "  " } else { "│ " };
                            }
                        }
                        s += "─ ";
                    }
                    s
                };
                let line = line + &line_meta.0.last().unwrap();
                let line = if line.len() > size.0 {
                    line.split_at(size.0).0.to_string()
                } else {
                    line
                };
                let len = line.chars().collect::<Vec<_>>().len();
                queue!(stdout, style::Print(line)).unwrap();
                for _ in 0..(size.0 - len) {
                    queue!(stdout, style::Print(" ".to_string())).unwrap();
                }
                queue!(stdout, cursor::MoveToColumn(0), cursor::MoveToNextLine(1)).unwrap();
                i += 1;
            }
        }
    }
}

#[derive(PartialEq, PartialOrd, Clone, Copy)]
enum PathType {
    File,
    Directory,
    SymLink,
    None,
}

struct Path(String, PathType, Rc<RefCell<bool>>, Vec<Box<Self>>);

impl PartialEq for Path {
    fn eq(&self, s: &Self) -> bool {
        self.0 == s.0
    }
}

impl PartialOrd for Path {
    fn partial_cmp(&self, s: &Self) -> Option<Ordering> {
        if self.1 == PathType::None || s.1 == PathType::None {
            None
        } else {
            match self.1.partial_cmp(&s.1) {
                Some(o) => match o {
                    Ordering::Less => Some(Ordering::Greater),
                    Ordering::Equal => Some(self.0.cmp(&s.0)),
                    Ordering::Greater => Some(Ordering::Less),
                },
                None => None,
            }
        }
    }
}

impl Eq for Path {}

impl Ord for Path {
    fn cmp(&self, other: &Self) -> Ordering {
        self.partial_cmp(other).unwrap()
    }
}

struct Filesystem {
    root: String,
    path_cache: Vec<Box<Path>>,
    showing_start: usize,
}

fn traverse(path: &str, path_local: &mut Vec<String>) -> Vec<Box<Path>> {
    let mut res = Vec::new();
    let list = if let Ok(l) = fs::read_dir(path) {
        l
    } else {
        return res;
    };
    for entry in list {
        let entry = entry.unwrap();
        path_local.push(entry.file_name().into_string().unwrap());
        let ptype = if entry.path().is_file() {
            PathType::File
        } else if entry.path().is_dir() {
            PathType::Directory
        } else if entry.path().is_symlink() {
            PathType::SymLink
        } else {
            PathType::None
        };
        res.push(Box::new(Path(
            entry.file_name().into_string().unwrap(),
            ptype,
            Rc::new(RefCell::new(false)),
            traverse(
                &(path.to_string() + "/" + &entry.file_name().into_string().unwrap()),
                path_local,
            ),
        )));
        let _ = path_local.pop();
    }
    res.sort();
    res
}

impl Filesystem {
    pub fn new(root: String) -> Self {
        let mut res = Filesystem {
            root,
            path_cache: Vec::new(),
            showing_start: 0,
        };
        res.traverse_fs();
        res.path_cache.sort();
        res
    }

    pub fn traverse_fs(&mut self) {
        let mut path_local = Vec::new();
        for entry in fs::read_dir(&self.root).unwrap() {
            let entry = entry.unwrap();
            path_local.push(entry.file_name().into_string().unwrap());
            let ptype = if entry.path().is_file() {
                PathType::File
            } else if entry.path().is_dir() {
                PathType::Directory
            } else if entry.path().is_symlink() {
                PathType::SymLink
            } else {
                PathType::None
            };
            self.path_cache.push(Box::new(Path(
                entry.file_name().into_string().unwrap(),
                ptype,
                Rc::new(RefCell::new(false)),
                traverse(
                    &(self.root.clone() + "/" + &entry.file_name().into_string().unwrap()),
                    &mut path_local,
                ),
            )));
            let _ = path_local.pop();
        }
    }

    pub fn fold_unfold(&self, path: &[String], cache: Option<&Vec<Box<Path>>>) {
        let cache = if let Some(c) = cache {
            c
        } else {
            &self.path_cache
        };
        if path.len() == 1 {
            for entry in cache {
                if entry.0 == path[0] {
                    let b = *entry.2.borrow();
                    *entry.2.borrow_mut() = !b;
                    break;
                }
            }
        } else {
            for entry in cache {
                if entry.0 == path[0] {
                    self.fold_unfold(&path[1..], Some(&entry.3));
                    break;
                }
            }
        }
    }

    pub fn iter(&self, max: usize) -> FilesystemIterator {
        let mut res = Vec::new();
        generate_meta_list(&self.path_cache, &mut res, 0, &mut vec![], &mut vec![]);
        let res = res[self.showing_start..].to_vec();
        let res = res[..max.min(res.len())].to_vec();
        FilesystemIterator { inner: res }
    }
}

fn generate_meta_list(
    paths: &Vec<Box<Path>>,
    res: &mut Vec<PathMeta>,
    depth: usize,
    cur_path: &mut Vec<String>,
    endflg_path: &mut Vec<bool>,
) {
    let mut c = 0;
    let l = paths.len();
    for path in paths {
        let Path(name, ptype, unfolded, directory) = path.as_ref();
        cur_path.push(name.clone());
        endflg_path.push(c + 1 == l);
        res.push((
            cur_path.clone(),
            *ptype,
            *unfolded.borrow(),
            depth,
            endflg_path.clone(),
        ));
        if *unfolded.borrow() {
            generate_meta_list(directory, res, depth + 1, cur_path, endflg_path);
        }
        let _ = cur_path.pop();
        let _ = endflg_path.pop();
        c += 1;
    }
}

type PathMeta = (Vec<String>, PathType, bool, usize, Vec<bool>);

struct FilesystemIterator {
    inner: Vec<PathMeta>,
}

impl Iterator for FilesystemIterator {
    type Item = PathMeta;

    fn next(&mut self) -> Option<Self::Item> {
        if self.inner.len() == 0 {
            None
        } else {
            Some(self.inner.remove(0))
        }
    }
}
