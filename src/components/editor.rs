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
                    if self.cursor.1 < *self.last_rh.lock().unwrap() {
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
                        let linelen = f.blocking_read().len_of_line(shst + self.cursor.1 - 1);
                        if linelen < lnst {
                            0
                        } else {
                            linelen - lnst
                        }
                    } else {
                        0
                    };
                    if self.cursor.0 > l {
                        self.cursor.0 = l;
                    } else {
                        self.cursor.0 -= 1;
                    }
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
                if !file.blocking_read().eof(sst) {
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
                    let mut lining = line.clone();
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
                            displaying
				.iter()
				.collect::<String>()
				.on(Color::Rgb {r: 0x38, g: 0x38, b: 0x58}),
                        );
                    } else {
                        renderer.set_section(
                            0,
                            linen,
                            displaying
				.iter()
				.collect::<String>()
				.on(Color::Rgb {r: 0x10, g: 0x10, b: 0x20}),
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
                    l.on(Color::Rgb {r: 0x10, g: 0x10, b: 0x20}),
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
    buffer: Vec<Vec<char>>,
    showing_start: usize,
    showing_length: usize,
    line_start: usize,
}

impl Editing {
    pub fn new(path: Vec<String>) -> Self {
        let mut res = Editing {
            path,
            buffer: vec![Vec::new()],
            showing_start: 1,
            showing_length: 0,
            line_start: 0,
        };
        res.load();
        res
    }

    pub fn load(&mut self) {
        let mut path = String::new();
	for p in &self.path {
	    path += "/";
	    path += &p;
	}
	let file = OpenOptions::new().read(true).open(&path).unwrap();
	let mut reader = BufReader::new(file);
	loop {
	    let mut line = Vec::new();
	    let l = match reader.read_until(b'\n', &mut line) {
		Ok(n) => n,
		Err(_) => 0,
	    };
	    if l == 0 {
		break;
	    }
	    let line = String::from_utf8(line).unwrap().chars().collect::<Vec<char>>();
	    self.buffer.push(line);
	}
    }

    pub fn len_of_line(&self, line: usize) -> usize {
        if let Some(l) = self.buffer.get(line) {
            UnicodeWidthStr::width(l.iter().collect::<String>().as_str())
        } else {
            0
        }
    }

    pub fn get(&mut self) -> Vec<Vec<char>> {
        let mut res = Vec::new();
        for i in self.showing_start..(self.showing_start + self.showing_length) {
	    if i >= self.buffer.len() {
		break;
	    }
	    res.push(self.buffer[i].clone());
	}
        res
    }

    pub fn path(&self) -> &Vec<String> {
        &self.path
    }

    pub fn eof(&self, cursor: usize) -> bool {
	self.showing_start + cursor > self.buffer.len()
    }
}
