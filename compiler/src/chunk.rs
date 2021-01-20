use std::fmt::{Display, Formatter, Result};

pub struct Chunk {
    name: String,
    data: Vec<(usize, String)>,
}

impl Chunk {
    pub fn new(name: String) -> Self {
        let data = vec![];
        Self { name, data }
    }

    pub fn insert_line(&mut self, line: String, id: usize) {
        self.data.push((id, line));
    }

    pub fn name(&self) -> &String {
        &self.name
    }

    pub fn data(self) -> Vec<(usize, String)> {
        self.data
    }
}

impl Display for Chunk {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        writeln!(f, ".{}", self.name)?;

        for (id, line) in self.data.iter() {
            writeln!(f, "{:>3} : {}", id, line)?;
        }

        Ok(())
    }
}
