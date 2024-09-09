pub struct IdGenerator {
    pub next_id: usize,
}

impl IdGenerator {
    pub fn new() -> Self {
        IdGenerator { next_id: 0 }
    }

    pub fn gen(&mut self) -> usize {
        let res = self.next_id;
        self.next_id += 1;

        res
    }

    pub fn peek(&self) -> usize {
        self.next_id
    }
}
