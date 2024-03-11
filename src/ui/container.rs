pub struct Container {
    x: usize,
    y: usize,
    width: usize,
    height: usize,
}

impl Container {
    pub fn new(x: usize, y: usize, width: usize, height: usize) -> Self {
        Container {
	    x, y, width, height,
	}
    }
    pub fn debug() {}
}
