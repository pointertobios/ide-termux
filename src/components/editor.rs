use crossterm::style::Stylize;
use std::{
    collections::HashMap,
    fs::OpenOptions,
    io::{BufRead, BufReader},
    iter,
    process::exit,
    sync::{Arc, RwLock},
};
use tokio::sync::{mpsc::Receiver, RwLock as AsyncRwLock};
use unicode_width::UnicodeWidthChar;

use crate::{
    components::component::Component,
    named_pipe::{NamedPipe, PipeObject},
    renderer::Renderer,
    ui::{
        container::{Container, ContainerType},
        framework::Framework,
    },
};

enum EditorMode {
    Command,
    Edit,
}

pub struct Editor {
    container: Arc<RwLock<Container>>,
    id: usize,
    file: Option<Arc<AsyncRwLock<Editing>>>,
    mode: EditorMode,
    /// 接收ProjectViewer发送的PipeObject::Editing(Editing)
    ///
    /// 总是渲染管道中最新发送的Editing对象
    file_open_receiver: Arc<AsyncRwLock<Receiver<PipeObject>>>,
}

impl Editor {
    pub fn new(id: usize) -> Arc<RwLock<Self>> {
        let container = Container::new(&("Editor".to_string() + &id.to_string()), None);
        let container = Arc::new(RwLock::new(container));
        let res = Arc::new(RwLock::new(Editor {
            container,
            id,
            file: None,
            mode: EditorMode::Command,
            file_open_receiver: NamedPipe::open_receiver(format!("FileOpen{}", id)),
        }));
        res.read()
            .unwrap()
            .container
            .write()
            .unwrap()
            .set_type(ContainerType::Editor(Arc::clone(&res)));
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

impl Component for Editor {
    fn bind_to(
        &mut self,
        framework: &mut Framework,
    ) -> Result<(), Box<dyn FnOnce(Framework) -> !>> {
        match framework.add_container("/WorkArea/EditorArea", Arc::clone(&self.container)) {
            Ok(()) => Ok(()),
            Err(s) => Err(Box::new(move |fw| {
                drop(fw);
                println!("{}", s);
                exit(-1)
            })),
        }
    }

    fn render(&mut self, renderer: &Renderer) -> (bool, (usize, usize)) {
        if let Ok(PipeObject::Editing(edi)) = self.file_open_receiver.blocking_write().try_recv() {
            self.file = Some(edi);
        }
        let size = renderer.get_size();
        let focused = self.container.read().unwrap().focused();
        let title = if let Some(f) = &self.file {
            " ".to_string() + &f.blocking_read().path.last().unwrap().clone()
        } else {
            format!(" Editor {}", self.id)
        };
        // 标题
        if !focused {
            let title = title.chars().collect::<Vec<_>>();
            if size.0 == 1 {
                let mut title = if title.len() > size.1 {
                    title.split_at(size.1).0.to_vec()
                } else {
                    title
                };
                if title.len() < size.1 {
                    title.append(
                        &mut iter::repeat(' ')
                            .take(size.1 - title.len())
                            .collect::<Vec<_>>(),
                    );
                }
                for i in 0..title.len() {
                    renderer.set(0, i, title[i].white().on_dark_grey());
                }
            } else {
                let mut title = if title.len() > size.0 {
                    title.split_at(size.1).0.to_vec()
                } else {
                    title
                };
                if title.len() < size.0 {
                    title.append(
                        &mut iter::repeat(' ')
                            .take(size.0 - title.len())
                            .collect::<Vec<_>>(),
                    );
                }
                let title = String::from_iter(title.iter());
                renderer.set_section(0, 0, title.white().on_dark_grey());
            }
        } else {
            let title = title.chars().collect::<Vec<_>>();
            let mut title = if title.len() > size.0 {
                title.split_at(size.1).0.to_vec()
            } else {
                title
            };
            if title.len() < size.0 {
                title.append(
                    &mut iter::repeat(' ')
                        .take(size.0 - title.len())
                        .collect::<Vec<_>>(),
                );
            }
            let title = String::from_iter(title.iter());
            renderer.set_section(0, 0, title.dark_red().on_dark_blue());
        }
        // 内容
        let mut cursor_loc = (0, 1);
        if size.0 > 1 && size.1 > 1 {
            let mut linen = 1;
            if let Some(file) = &self.file {
                file.blocking_write().showing_length = size.1;
                for line in file.blocking_write().get() {
                    let mut lining = line.origin_content;
                    if !lining.is_empty() && *lining.last().unwrap() == '\n' {
                        lining.pop();
                    }
                    let mut displaying = Vec::new();
                    let mut rawl = 0;
                    while rawl < size.0 {
                        if lining.is_empty() {
                            break;
                        }
                        let ch = lining.remove(0);
                        rawl += UnicodeWidthChar::width(ch).unwrap();
                        displaying.push(ch);
                    }
                    if rawl < size.0 {
                        displaying.append(
                            &mut iter::repeat(' ').take(size.0 - rawl).collect::<Vec<char>>(),
                        );
                    }
                    renderer.set_section(0, linen, displaying.iter().collect::<String>().reset());
                    linen += 1;
                }
            }
            while linen < size.1 {
                let l = iter::repeat(' ').take(size.0).collect::<Vec<_>>();
                let l = String::from_iter(&mut l.iter());
                renderer.set_section(0, linen, l.reset());
                linen += 1;
            }
        }
        cursor_loc.0 += renderer.x;
        cursor_loc.1 += renderer.y;
        (true, cursor_loc)
    }
}

pub struct Editing {
    path: Vec<String>,
    buffer: HashMap<usize, LineDiff>,
    showing_start: usize,
    showing_length: usize,
}

impl Editing {
    pub fn new(path: Vec<String>) -> Self {
        let mut res = Editing {
            path,
            buffer: HashMap::new(),
            showing_start: 1,
            showing_length: 0,
        };
        res.load(1, 200);
        res
    }

    pub fn load(&mut self, start: usize, count: usize) {
        if start < 1 {
            return;
        }
        let mut path = String::new();
        for entry in &self.path {
            path += "/";
            path += entry;
        }
        let mut i = 0;
        for line in LineDiff::gen_lines(path, start, count) {
            self.buffer.insert(start + i, line);
            i += 1;
        }
    }

    pub fn get(&mut self) -> Vec<LineDiff> {
        let mut res = Vec::new();
        if self.showing_start < 1 {
            return res;
        }
        for i in self.showing_start..self.showing_start + self.showing_length {
            let b = self.buffer.contains_key(&i);
            if b {
                // TODO 为了简便而这样写，后面需要改进（这个函数所调用的LineDiff::gen_lines函数是从文件开头开始扫描，而不是在适当的时机直接跳过）
                self.load(i, 1);
            }
            let line = if let Some(l) = self.buffer.get(&i) {
                l.clone()
            } else {
                break;
            };
            // println!("{:?}", line);
            res.push(line);
        }
        res
    }

    pub fn path(&self) -> &Vec<String> {
        &self.path
    }
}

#[derive(Clone, Debug)]
pub struct LineDiff {
    origin_offset: usize,
    /// 对于一个文件中原有的行，这个vec总是以换行结尾，即使是空行
    /// 新增行此成员为空
    origin_content: Vec<char>,
    /// 编辑中的差异信息
    /// .0：更改的位置
    /// .1：增加的字符vec
    /// .2：删除插入点后的字符数量
    ///
    /// 说明：当前行尾的换行若被移除，说明下一行合并到这一行
    changes: Vec<(usize, Vec<char>, usize)>,
}

impl LineDiff {
    pub fn gen_lines(path: String, start_line: usize, count: usize) -> Vec<LineDiff> {
        // 路径是启动时加载目录所获得，忽略返回Err()的情况
        let file = OpenOptions::new().read(true).open(&path).unwrap();
        let mut reader = BufReader::new(file);
        let mut res = Vec::new();
        let mut line = 1;
        let mut bytes = 0;
        // 跳过start_line前的行
        for i in 0..start_line - 1 {
            let l = match reader.skip_until(b'\n') {
                Ok(n) => n,
                Err(_) => 0,
            };
            if i + 1 != start_line - 1 && l == 0 {
                return res;
            }
            bytes += l;
            line += 1;
        }
        // 构造需要显示的行
        for i in 0..count {
            let mut buf = Vec::new();
            let l = match reader.read_until(b'\n', &mut buf) {
                Ok(n) => n,
                Err(_) => 0,
            };
            if i + 1 != count && l == 0 {
                break;
            }
            res.push(LineDiff {
                origin_offset: bytes,
                origin_content: String::from_utf8(buf)
                    .unwrap()
                    .chars()
                    .collect::<Vec<char>>(),
                changes: Vec::new(),
            });
            bytes += l;
            line += 1;
        }
        res
    }
}
