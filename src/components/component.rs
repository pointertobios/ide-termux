use std::io::Stdout;

use crate::ui::framework::Framework;

pub trait Component {
    fn bind_to(&mut self, framework: &mut Framework)
        -> Result<(), Box<dyn FnOnce(Framework) -> !>>;
    fn render(&self, offset: (usize, usize), size: (usize, usize), stdout: &mut Stdout);
}
