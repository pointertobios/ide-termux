mod ui;

use ui::framework::Framework;

use crossterm::event::{
    read,
    Event,
    KeyEvent,
    KeyEventKind,
    KeyModifiers,
    KeyCode,
};

pub fn run() -> std::io::Result<()> {
    let mut framework = Framework::new();
    loop {
        framework.render();
        match read()? {
	    Event::Key(KeyEvent {
	        kind: KeyEventKind::Press | KeyEventKind::Repeat,
		modifiers: KeyModifiers::CONTROL,
		code,
		..
	    }) => {
	        match code {
		    KeyCode::Char('d') => {
		        break;
		    }
		    _ => (),
		}
	    }
	    Event::Resize(width, height) =>
	        framework.set_size(width as usize, height as usize),
	    _ => (),
	}
    }
    Ok(())
}