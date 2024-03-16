use std::sync::{Arc, RwLock};

use crate::{renderer::Renderer, ui::framework::Framework};

pub trait Component {
    fn bind_to(&mut self, framework: &mut Framework)
        -> Result<(), Box<dyn FnOnce(Framework) -> !>>;
    fn render(&self, renderer: Arc<RwLock<Renderer>>) -> (bool, (usize, usize));
}
