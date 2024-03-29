use crossterm::{
    event::{Event, KeyCode, KeyEvent, KeyModifiers},
    style::{Color, Stylize},
};
use std::{
    collections::HashMap,
    fs::OpenOptions,
    io::{BufRead, BufReader},
    iter,
    process::exit,
    sync::{Arc, Mutex, RwLock},
};
use tokio::sync::{mpsc::Receiver, RwLock as AsyncRwLock};
use unicode_width::{UnicodeWidthChar, UnicodeWidthStr};

use crate::{
    components::component::Component,
    named_pipe::{NamedPipe, PipeObject},
    renderer::Renderer,
    ui::{
        container::{Container, ContainerType},
        framework::Framework,
    },
};

#[derive(Clone, Copy)]
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
    last_rh: Arc<Mutex<usize>>,
    cursor: (usize, usize),
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
            last_rh: Arc::new(Mutex::new(0)),
            cursor: (0, 1),
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
            .set_handler(Box::new(move |event, contsize| match event {
                Event::Key(KeyEvent {
                    code,
                    modifiers: KeyModifiers::NONE,
                    ..
                }) => match code {
                    KeyCode::Up => {
                        res_ref.write().unwrap().cursor_up(contsize);
                    }
                    KeyCode::Down => {
                        res_ref.write().unwrap().cursor_down(contsize);
                    }
                    KeyCode::Left => {
                        res_ref.write().unwrap().cursor_left(contsize);
                    }
                    KeyCode::Right => {
                        res_ref.write().unwrap().cursor_right(contsize);
                    }
                    KeyCode::Esc => {
                        res_ref.write().unwrap().mode = EditorMode::Command;
                    }
                    KeyCode::Char('e') => {
                        res_ref.write().unwrap().mode = EditorMode::Edit;
                    }
                    _ => (),
                },
                _ => (),
            }));
        res
    }

    fn cursor_up(&mut self, _contsize: (usize, usize)) {
        match self.mode {
            EditorMode::Command => {
                self.scroll_up(1);
            }
            EditorMode::Edit => {
                if self.cursor.1 > 1 {
                    self.cursor.1 -= 1;
                } else {
                    self.scroll_up(1);
                }
            }
        }
    }

    fn cursor_down(&mut self, contsize: (usize, usize)) {
        match self.mode {
            EditorMode::Command => {
                self.scroll_down(1);
            }
            EditorMode::Edit => {
                if self.cursor.1 < contsize.1 - 1 {
                    if self.cursor.1 + 1 < *self.last_rh.lock().unwrap() - 1 {
                        self.cursor.1 += 1;
                    }
                } else {
                    self.scroll_down(1);
                }
            }
        }
    }

    fn cursor_left(&mut self, contsize: (usize, usize)) {
        match self.mode {
            EditorMode::Command => {
                self.scroll_left(1);
                if self.cursor.0 < contsize.0 {
                    self.cursor.0 += if self.cursor.0 + 1 == contsize.0 {
                        1
                    } else {
                        2
                    };
                }
            }
            EditorMode::Edit => {
                if self.cursor.0 > 0 {
                    let l = if let Some(f) = &self.file {
                        let shst = f.blocking_read().showing_start;
                        let lnst = f.blocking_read().line_start;
                        f.blocking_read().len_of_line(shst + self.cursor.1 - 1) - lnst
                    } else {
                        0
                    };
                    if self.cursor.0 > l {
                        self.cursor.0 = l;
                    }
                    self.cursor.0 -= 1;
                } else {
                    self.scroll_left(1);
                }
            }
        }
    }

    fn cursor_right(&mut self, contsize: (usize, usize)) {
        match self.mode {
            EditorMode::Command => {
                self.scroll_right(1);
                if self.cursor.0 > 0 {
                    self.cursor.0 -= if self.cursor.0 == 1 { 1 } else { 2 };
                }
            }
            EditorMode::Edit => {
                let l = if let Some(f) = &self.file {
                    let shst = f.blocking_read().showing_start;
                    let lnst = f.blocking_read().line_start;
                    f.blocking_read().len_of_line(shst + self.cursor.1 - 1) - lnst
                } else {
                    0
                };
                if self.cursor.0 < contsize.0 - 1 {
                    if self.cursor.0 < l {
                        self.cursor.0 += 1;
                    }
                } else {
                    if let Some(f) = &self.file {
                        if self.cursor.0 < l - 1 {
                            self.scroll_right(1);
                        }
                    }
                }
            }
        }
    }

    fn scroll_up(&self, count: usize) {
        for _ in 0..count {
            if let Some(file) = &self.file {
                if file.blocking_read().showing_start > 1 {
                    file.blocking_write().showing_start -= 1;
                }
            }
        }
    }

    fn scroll_down(&self, count: usize) {
        for _ in 0..count {
            if let Some(file) = &self.file {
                let sst = file.blocking_read().showing_start;
                if !file.blocking_read().eof(sst + 3) {
                    file.blocking_write().showing_start += 1;
                }
            }
        }
    }

    fn scroll_left(&self, count: usize) {
        for _ in 0..count {
            if let Some(file) = &self.file {
                if file.blocking_read().line_start > 0 {
                    file.blocking_write().line_start -= 2;
                }
            }
        }
    }

    fn scroll_right(&self, count: usize) {
        for _ in 0..count {
            if let Some(file) = &self.file {
                file.blocking_write().line_start += 2;
            }
        }
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
        let mode = match self.mode {
            EditorMode::Command => "Command".to_string(),
            EditorMode::Edit => "Editing".to_string(),
        };
        // 标题
        if !focused {
            let title = title.chars().collect::<Vec<_>>();
            let mut mode = mode.chars().collect::<Vec<_>>();
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
                let mut title = if title.len() > size.0 - mode.len() - 1 {
                    title.split_at(size.1 - mode.len() - 1).0.to_vec()
                } else {
                    title
                };
                if title.len() < size.0 - mode.len() - 1 {
                    title.append(
                        &mut iter::repeat(' ')
                            .take(size.0 - title.len() - mode.len())
                            .collect::<Vec<_>>(),
                    );
                }
                title.append(&mut mode);
                let title = String::from_iter(title.iter());
                renderer.set_section(0, 0, title.white().on_dark_grey());
            }
        } else {
            let title = title.chars().collect::<Vec<_>>();
            let mut mode = mode.chars().collect::<Vec<_>>();
            let mut title = if title.len() > size.0 - mode.len() - 1 {
                title.split_at(size.1 - mode.len() - 1).0.to_vec()
            } else {
                title
            };
            if title.len() < size.0 - mode.len() - 1 {
                title.append(
                    &mut iter::repeat(' ')
                        .take(size.0 - title.len() - mode.len())
                        .collect::<Vec<_>>(),
                );
            }
            title.append(&mut mode);
            let title = String::from_iter(title.iter());
            renderer.set_section(0, 0, title.dark_red().on_dark_blue());
        }
        // 内容
        let mut cursor_loc = self.cursor;
        if size.0 > 1 && size.1 > 1 {
            let mut linen = 1;
            if let Some(file) = &self.file {
                file.blocking_write().showing_length = size.1;
                let lnst = file.blocking_read().line_start;
                for line in file.blocking_write().get() {
                    let linelen = line.len();
                    if linen == cursor_loc.1 {
                        if linelen > lnst && cursor_loc.0 > linelen - lnst {
                            cursor_loc.0 = linelen - lnst;
                        } else if linelen <= lnst && cursor_loc.0 > 0 {
                            cursor_loc.0 = 0;
                        }
                    }
                    let mut lining = line.origin_content.clone();
                    if !lining.is_empty() && *lining.last().unwrap() == '\n' {
                        lining.pop();
                    }
                    let mut displaying = Vec::new();
                    let mut rawl = 0;
                    while !lining.is_empty() && rawl < lnst {
                        let ch = lining.remove(0);
                        rawl += UnicodeWidthChar::width(ch).unwrap();
                    }
                    rawl = 0;
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
                    if linen == self.cursor.1 {
                        renderer.set_section(
                            0,
                            linen,
                            displaying.iter().collect::<String>().on(Color::Rgb {
                                r: 0x38,
                                g: 0x38,
                                b: 0x58,
                            }),
                        );
                    } else {
                        renderer.set_section(
                            0,
                            linen,
                            displaying.iter().collect::<String>().on(Color::Rgb {
                                r: 0x10,
                                g: 0x10,
                                b: 0x20,
                            }),
                        );
                    }
                    linen += 1;
                }
            }
            *self.last_rh.lock().unwrap() = linen;
            while linen < size.1 {
                let l = iter::repeat(' ').take(size.0).collect::<Vec<_>>();
                let l = String::from_iter(&mut l.iter());
                renderer.set_section(
                    0,
                    linen,
                    l.on(Color::Rgb {
                        r: 0x10,
                        g: 0x10,
                        b: 0x20,
                    }),
                );
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
    line_start: usize,
}

impl Editing {
    pub fn new(path: Vec<String>) -> Self {
        let mut res = Editing {
            path,
            buffer: HashMap::new(),
            showing_start: 1,
            showing_length: 0,
            line_start: 0,
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

    pub fn len_of_line(&self, line: usize) -> usize {
        if let Some(l) = self.buffer.get(&line) {
            l.len()
        } else {
            0
        }
    }

    pub fn get(&mut self) -> Vec<LineDiff> {
        let mut res = Vec::new();
        if self.showing_start < 1 {
            return res;
        }
        for i in self.showing_start..self.showing_start + self.showing_length {
            let b = self.buffer.contains_key(&i);
            if !b {
                // TODO 为了简便而这样写，后面需要改进（这个函数所调用的LineDiff::gen_lines函数是从文件开头开始扫描，而不是在适当的时机直接跳过）
                self.load(i, 1);
            }
            let line = if let Some(l) = self.buffer.get(&i) {
                l.clone()
            } else {
                break;
            };
            res.push(line);
        }
        res
    }

    pub fn path(&self) -> &Vec<String> {
        &self.path
    }

    pub fn eof(&self, line: usize) -> bool {
        let mut path = String::new();
        for entry in &self.path {
            path += "/";
            path += entry;
        }
        LineDiff::gen_lines(path, line - 1, 1).is_empty()
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
        }
        res
    }

    pub fn len(&self) -> usize {
        UnicodeWidthStr::width(String::from_iter(self.origin_content.iter()).as_str())
    }
}
