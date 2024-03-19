#![feature(never_type)]

mod components;
mod named_pipe;
mod renderer;
mod ui;

use components::{
    areas::{EditorArea, WorkArea},
    component::Component,
    editor::Editor,
    project_viewer::ProjectViewer,
    terminal::Terminal,
};
use ui::{
    container::{Container, ContainerType},
    framework::Framework,
    ChangeFocusEvent,
};

use crossterm::event::{read, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};

pub fn run() -> std::io::Result<()> {
    let mut framework = Framework::new();

    let terminal = Terminal::new();
    if let Err(f) = terminal.write().unwrap().bind_to(&mut framework) {
        f(framework);
    }

    let work_area = WorkArea::new();
    if let Err(f) = work_area.write().unwrap().bind_to(&mut framework) {
        f(framework);
    }

    let editor_area = EditorArea::new();
    if let Err(f) = editor_area.write().unwrap().bind_to(&mut framework) {
        f(framework);
    }

    let project_viewer = ProjectViewer::new();
    if let Err(f) = project_viewer.write().unwrap().bind_to(&mut framework) {
        f(framework);
    }

    let editor0 = Editor::new(0);
    if let Err(f) = editor0.write().unwrap().bind_to(&mut framework) {
        f(framework);
    }

    let editor1 = Editor::new(1);
    if let Err(f) = editor1.write().unwrap().bind_to(&mut framework) {
        f(framework);
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
                modifiers,
                code,
                state,
            }) => {
                if modifiers.contains(KeyModifiers::CONTROL)
                    && modifiers.contains(KeyModifiers::ALT)
                {
                    match code {
                        KeyCode::Char('d') => {
                            break;
                        }
                        KeyCode::Up => {
                            framework.dispatch(ui::Event::ChangeFocus(ChangeFocusEvent::Up))
                        }
                        KeyCode::Down => {
                            framework.dispatch(ui::Event::ChangeFocus(ChangeFocusEvent::Down))
                        }
                        KeyCode::Left => {
                            framework.dispatch(ui::Event::ChangeFocus(ChangeFocusEvent::Left))
                        }
                        KeyCode::Right => {
                            framework.dispatch(ui::Event::ChangeFocus(ChangeFocusEvent::Right))
                        }
                        _ => (),
                    }
                } else {
                    framework.dispatch(ui::Event::Crossterm(Event::Key(KeyEvent {
                        code,
                        modifiers,
                        kind: KeyEventKind::Press,
                        state,
                    })));
                }
            }
            Event::Resize(width, height) => framework.set_size(width as usize, height as usize),
            event => framework.dispatch(ui::Event::Crossterm(event)),
        }
    }
    Ok(())
}
