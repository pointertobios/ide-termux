use rand::{rngs::ThreadRng, Rng};

pub struct PseudoMultithreading<T>
where
    T: Send + 'static,
{
    tasks: Vec<Box<dyn FnOnce() -> T>>,
    rng: ThreadRng,
}

impl<T: Send + 'static> PseudoMultithreading<T> {
    pub fn new() -> Self {
        PseudoMultithreading {
            tasks: Vec::new(),
            rng: rand::thread_rng(),
        }
    }

    pub fn add(&mut self, f: Box<dyn FnOnce() -> T>) {
        self.tasks.push(f);
    }

    pub fn run(&mut self) {
        while !self.tasks.is_empty() {
            let r = self.rng.gen_range(0..self.tasks.len());
            let f = self.tasks.remove(r);
            f();
        }
    }
}
