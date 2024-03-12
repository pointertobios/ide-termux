mod ui;

use std::{
    process::exit,
    sync::{Arc, RwLock},
};

use ui::{
    container::{Container, ContainerType},
    framework::Framework,
    ChangeFocusEvent,
};

use crossterm::event::{read, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};

pub fn run() -> std::io::Result<()> {
    let mut framework = Framework::new();
    let root = Container::new_root(framework.get_size().0, framework.get_size().1, None);
    let root = Arc::new(RwLock::new(root));
    framework.set_container(Arc::clone(&root));
    //framework.set_focused_path("/WorkArea/ProjectViewer");
    framework.set_focused_path("/Terminal");

    let mut terminal_cont = Container::new(&"Terminal".to_string(), None);
    terminal_cont.set_type(ContainerType::Terminal);
    terminal_cont.focus();
    let terminal_cont = Arc::new(RwLock::new(terminal_cont));
    if let Err(s) = framework.add_container("/", terminal_cont) {
        drop(framework);
        println!("{}", s);
        exit(-1);
    }

    let mut workarea_cont = Container::new(&"WorkArea".to_string(), None);
    workarea_cont.set_type(ContainerType::Father {
        subconts: [None, None],
        vert_layout: false,
        all_own: true,
    });
    //workarea_cont.focus();
    let workarea_cont = Arc::new(RwLock::new(workarea_cont));
    if let Err(s) = framework.add_container("/", workarea_cont) {
        drop(framework);
        println!("{}", s);
        exit(-1);
    }

    let mut editorarea_cont = Container::new(&"EditorArea".to_string(), None);
    editorarea_cont.set_type(ContainerType::Father {
        subconts: [None, None],
        vert_layout: true,
        all_own: false,
    });
    //editorarea_cont.focus();
    let editorarea_cont = Arc::new(RwLock::new(editorarea_cont));
    if let Err(s) = framework.add_container("/WorkArea", editorarea_cont) {
        drop(framework);
        println!("{}", s);
        exit(-1);
    }

    let mut projv_cont = Container::new(&"ProjectViewer".to_string(), None);
    projv_cont.set_type(ContainerType::ProjectViewer);
    //projv_cont.focus();
    let projv_cont = Arc::new(RwLock::new(projv_cont));
    if let Err(s) = framework.add_container("/WorkArea", projv_cont) {
        drop(framework);
        println!("{}", s);
        exit(-1);
    }

    let mut editor_0 = Container::new(&"Editor0".to_string(), None);
    editor_0.set_type(ContainerType::Editor);
    //editor_0.focus();
    let editor_0 = Arc::new(RwLock::new(editor_0));
    if let Err(s) = framework.add_container("/WorkArea/EditorArea", editor_0) {
        drop(framework);
        println!("{}", s);
        exit(-1);
    }

    let mut editor_1 = Container::new(&"Editor1".to_string(), None);
    editor_1.set_type(ContainerType::Editor);
    let editor_1 = Arc::new(RwLock::new(editor_1));
    if let Err(s) = framework.add_container("/WorkArea/EditorArea", editor_1) {
        drop(framework);
        println!("{}", s);
        exit(-1);
    }

    framework.set_adjacy(
        "/Terminal".to_string(),
        (
            Some("/WorkArea/ProjectViewer".to_string()),
            None,
            None,
            None,
        ),
    );
    framework.set_adjacy(
        "/WorkArea/ProjectViewer".to_string(),
        (
            None,
            Some("/Terminal".to_string()),
            None,
            Some("/WorkArea/EditorArea/Editor0".to_string()),
        ),
    );
    framework.set_adjacy(
        "/WorkArea/EditorArea/Editor0".to_string(),
        (
            Some("/WorkArea/EditorArea/Editor1".to_string()),
            Some("/Terminal".to_string()),
            Some("/WorkArea/ProjectViewer".to_string()),
            None,
        ),
    );
    framework.set_adjacy(
        "/WorkArea/EditorArea/Editor1".to_string(),
        (
            None,
            Some("/WorkArea/EditorArea/Editor0".to_string()),
            Some("/WorkArea/ProjectViewer".to_string()),
            None,
        ),
    );

    let fsize = framework.get_size();
    framework.set_size(fsize.0, fsize.1);
    loop {
        framework.render();
        match read()? {
            Event::Key(KeyEvent {
                kind: KeyEventKind::Press | KeyEventKind::Repeat,
                modifiers: KeyModifiers::ALT,
                code,
                ..
            }) => match code {
                KeyCode::Char('d') => {
                    break;
                }
                KeyCode::Up => framework.dispatch(ui::Event::ChangeFocus(ChangeFocusEvent::Up)),
                KeyCode::Down => framework.dispatch(ui::Event::ChangeFocus(ChangeFocusEvent::Down)),
                KeyCode::Left => framework.dispatch(ui::Event::ChangeFocus(ChangeFocusEvent::Left)),
                KeyCode::Right => {
                    framework.dispatch(ui::Event::ChangeFocus(ChangeFocusEvent::Right))
                }
                _ => (),
            },
            Event::Resize(width, height) => framework.set_size(width as usize, height as usize),
            _ => (),
        }
    }
    Ok(())
}
