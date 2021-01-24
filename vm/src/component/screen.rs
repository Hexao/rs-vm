use crate::component::memory_io::*;
use std::cmp::Ordering;
use super::memory_io;

pub struct Screen {
    height: usize,
    width: usize,
    row: usize,
}

impl Screen {
    pub const CLEAR_LINE: u8 = 0xFE;
    pub const CLEAR: u8 = 0xFF;

    pub fn new(width: usize, height: usize) -> Self {
        Self { height, width, row: 1 }
    }

    /// place the cursor in the cell (x, y).
    /// `x` and `y` must be greater than one
    fn move_to(&mut self, x: usize, y: usize) {
        self.move_row(y);
        self.move_col(x);
    }

    /// move the curson in the column
    /// `col` without changing the row
    fn move_col(&mut self, col: usize) {
        print!("\x1b[{}G", col);
    }

    fn move_row(&mut self, row: usize) {
        match row.cmp(&self.row) {
            Ordering::Greater => print!("\x1B[{}E", row - self.row),
            Ordering::Less => print!("\x1B[{}F", self.row - row),
            _ => (),
        }

        self.row = row;
    }

    /// clear the entier screen then place
    /// the cursor at the top left corner
    fn clear(&mut self) {
        print!("\x1B[2J"); // clear screen
        print!("\x1B[1;1H"); // move to top left
        self.row = 1;
    }

    /// clear the current line then place the
    /// cursor at the beginning of the line
    fn clear_line(&mut self) {
        print!("\x1B[1K");
        self.move_col(1);
    }

    fn exec_code(&mut self, code: u8) {
        match code {
            Self::CLEAR_LINE => self.clear_line(),
            Self::CLEAR => self.clear(),
            _ => (),
        }
    }
}

impl MemoryIO for Screen {
    fn get_memory_at_u8(&self, _location: usize) -> Result<u8, MemoryError> {
        Ok(0)
    }

    fn get_memory_at_u16(&self, _location: usize) -> Result<u16, MemoryError> {
        Ok(0)
    }

    /// use the data for execute some specific instructions on the screen, ignore location
    fn set_memory_at_u8(&mut self, _location: usize, data: u8) -> Result<(), MemoryError> {
        self.exec_code(data);

        Ok(())
    }

    /// puts specific char on the screen. the char location is specified by `location`, obviously.
    /// the `data` will be split in two parts. The eights upper bits (`0xFF00`) will be used for
    /// execute some specific instructions on the screen. The reste of these bites (`0x00FF`) will
    /// be used to print one character on the screen
    fn set_memory_at_u16(&mut self, location: usize, data: u16) -> Result<(), MemoryError> {
        let x = location % self.width;
        let y = location / self.width;

        if y >= self.height {
            return Err(memory_io::MemoryError::OutOfBounds(location));
        }

        self.move_to(x + 1, y + 1);
        let code = (data >> 8) as u8;
        let character = std::char::from_u32((data & 0xFF) as u32).unwrap();

        self.exec_code(code);
        print!("{}", character);

        Ok(())
    }

    /// return the number of byte needed
    /// to write on every cell of the screen
    fn len(&self) -> usize {
        self.width * self.height
    }

    fn is_empty(&self) -> bool {
        self.len() > 0
    }
}
