use crate::{
    components::{component::Component, editor::Editing},
    named_pipe::{NamedPipe, PipeObject},
    renderer::Renderer,
    Container, ContainerType, Framework,
};
use crossterm::{
    cursor,
    event::{Event, KeyCode, KeyEvent, KeyEventKind, KeyEventState, KeyModifiers},
    queue,
    style::{self, Stylize},
};
use std::{
    cell::RefCell,
    cmp::Ordering,
    fs,
    io::Write,
    iter,
    process::exit,
    rc::Rc,
    sync::{Arc, RwLock},
};
use tokio::sync::{mpsc::Sender, RwLock as AsyncRwLock};

pub struct ProjectViewer {
    container: Arc<RwLock<Container>>,
    path: String,
    at_line: usize,
    fs: Filesystem,
    editor_stack: Vec<Arc<AsyncRwLock<Editing>>>,
    file_open_sender: [Arc<AsyncRwLock<Sender<PipeObject>>>; 2],
    move_focus_sender: Arc<AsyncRwLock<Sender<PipeObject>>>,
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
            editor_stack: Vec::new(),
            fs: Filesystem::new(path),
            file_open_sender: [
                NamedPipe::open_sender(String::from("FileOpen0")),
                NamedPipe::open_sender(String::from("FileOpen1")),
            ],
	    move_focus_sender: NamedPipe::open_sender(String::from("MoveFocusToEditor")),
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
                            } else if res_ref.read().unwrap().fs.showing_start > 0 {
                                res_ref.write().unwrap().fs.showing_start -= 1;
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
                            } else if contsize.1 - 1 < *res_ref.read().unwrap().fs.last_max.borrow()
                            {
                                res_ref.write().unwrap().fs.showing_start += 1;
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
                                // 展开目录
                                res_ref.write().unwrap().fs.fold_unfold(&meta.0, None);
                            } else {
                                // 打开文件
                                let file_path = res_ref.read().unwrap().path.clone();
                                let mut file_path = file_path
                                    .split("/")
                                    .map(|s| s.to_string())
                                    .collect::<Vec<String>>();
                                file_path.append(&mut meta.0.clone());
                                // editor stack是否存在这个文件
                                let ind = res_ref
                                    .read()
                                    .unwrap()
                                    .editor_stack
                                    .iter()
                                    .enumerate()
                                    .filter_map(|(i, edi)| {
                                        if *edi.blocking_read().path() == file_path {
                                            Some(i)
                                        } else {
                                            None
                                        }
                                    })
                                    .collect::<Vec<usize>>();
                                let editing = if ind.is_empty() {
                                    // 不存在则构造一个
                                    let editing = Editing::new(file_path);
                                    let editing = Arc::new(AsyncRwLock::new(editing));
                                    editing
                                } else {
                                    let ind = ind[0];
                                    let ed = res_ref.write().unwrap().editor_stack.remove(ind);
                                    ed
                                };
                                if let Some(ed) = res_ref.read().unwrap().editor_stack.last() {
                                    res_ref.read().unwrap().file_open_sender[1]
                                        .blocking_write()
                                        .try_send(PipeObject::Editing(Arc::clone(ed)))
                                        .unwrap();
                                }
                                res_ref
                                    .write()
                                    .unwrap()
                                    .editor_stack
                                    .push(Arc::clone(&editing));
                                res_ref.read().unwrap().file_open_sender[0]
                                    .blocking_read()
                                    .try_send(PipeObject::Editing(editing))
                                    .unwrap();
                            }
			    res_ref.read().unwrap().move_focus_sender.blocking_read().try_send(PipeObject::MoveFocus).unwrap();
                        }
                        _ => (),
                    },
                    Event::Resize(..) => {
                        if contsize.1 > 1 {
                            let at_line = res_ref.read().unwrap().at_line;
                            if at_line >= contsize.1 - 1 {
                                res_ref.write().unwrap().fs.showing_start +=
                                    at_line + 2 - contsize.1;
                                res_ref.write().unwrap().at_line = contsize.1 - 2;
                            }
                        }
                    }
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

    fn render(&mut self, renderer: &Renderer) -> (bool, (usize, usize)) {
        let size = renderer.get_size();
        let focused = self.container.read().unwrap().focused();
        let title = self.path.split("/").last().unwrap().to_string();
        let title = title.chars().collect::<Vec<_>>();
        let mut title = if title.len() > if size.0 == 1 { size.1 } else { size.0 } {
            title
                .split_at(if size.0 == 1 { size.1 } else { size.0 })
                .0
                .to_vec()
        } else {
            title
        };
        title.append(
            &mut iter::repeat(' ')
                .take(if size.0 == 1 { size.1 } else { size.0 } - title.len())
                .collect::<Vec<_>>(),
        );
        let mut titlev = title;
        let title = String::from_iter(titlev.iter());
        if !focused {
            if size.0 == 1 {
                let mut title = String::from("ProjViewer | ").chars().collect::<Vec<_>>();
                title.append(&mut titlev);
                for i in 0..title.len() {
                    renderer.set(0, i, title[i].white().on_dark_grey());
                }
            } else {
                renderer.set_section(0, 0, title.white().on_dark_grey());
            }
        } else {
            // 绘制标题
            renderer.set_section(0, 0, title.dark_red().on_dark_blue());
            // 绘制主体
            let mut linen = 1;
            for (path, ptype, open, depth, endflg_path) in self.fs.iter(size.1 - 1) {
                let mut s = String::new();
                for i in 0..depth {
                    s += if i == depth - 1 {
                        if ptype == PathType::Directory {
                            if *endflg_path.last().unwrap() {
                                "┕━"
                            } else {
                                "┝━"
                            }
                        } else {
                            if *endflg_path.last().unwrap() {
                                "╰─"
                            } else {
                                "├─"
                            }
                        }
                    } else {
                        if endflg_path[i + 1] {
                            "  "
                        } else {
                            "│ "
                        }
                    };
                }
                s += if ptype == PathType::Directory {
                    if open {
                        "┭ "
                    } else {
                        "╾ "
                    }
                } else if ptype == PathType::None {
                    "──"
                } else {
                    "─ "
                };
                s += &path.last().unwrap();
                let mut s = if s.chars().collect::<Vec<_>>().len() > size.0 {
                    s.chars().collect::<Vec<_>>().split_at(size.0).0.to_vec()
                } else {
                    s.chars().collect::<Vec<_>>()
                };
                s.append(&mut iter::repeat(' ').take(size.0 - s.len()).collect::<Vec<_>>());
                let s = String::from_iter(s.iter());
                renderer.set_section(
                    0,
                    linen,
                    if linen - 1 == self.at_line {
                        s.black().on_grey()
                    } else {
                        s.reset()
                    },
                );
                linen += 1;
            }
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
    last_max: Rc<RefCell<usize>>,
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
            last_max: Rc::new(RefCell::new(0)),
        };
        queue!(
            std::io::stdout(),
            cursor::MoveTo(0, 0),
            style::Print("加载项目...")
        )
        .unwrap();
        std::io::stdout().flush().unwrap();
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
        *self.last_max.borrow_mut() = res.len();
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
    if l == 0 {
        cur_path.push("".to_string());
        endflg_path.push(true);
        res.push((
            cur_path.clone(),
            PathType::None,
            false,
            depth,
            endflg_path.clone(),
        ));
        let _ = cur_path.pop();
        let _ = endflg_path.pop();
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
