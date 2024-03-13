use crossterm::event;

pub mod container;
pub mod framework;

pub enum Event {
    ChangeFocus(ChangeFocusEvent),
    Crossterm(event::Event),
}

pub enum ChangeFocusEvent {
    Up,
    Down,
    Left,
    Right,
}
