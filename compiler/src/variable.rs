#[derive(Debug)]
pub enum Type {
    None, U8, U16,
}

impl Type {
    pub fn type_len(&self) -> usize {
        match *self {
            Type::None => 0,
            Type::U8 => 1,
            Type::U16 => 2,
        }
    }
}

impl From<&str> for Type {
    fn from(string: &str) -> Self {
        match string.to_lowercase().as_str() {
            "u8" => Type::U8,
            "u16" => Type::U16,
            _ => Type::None,
        }
    }
}

#[derive(Debug)]
pub struct Var {
    data_type: Type,
    data: Vec<u8>,
    location: u16,
}

impl Var {
    pub fn get_type(&self) -> &Type {
        &self.data_type
    }

    pub fn set_type(&mut self, data_type: Type) {
        self.data_type = data_type;
    }

    pub fn type_len(&self) -> usize {
        self.data_type.type_len()
    }

    pub fn get_location(&self) -> &u16 {
        &self.location
    }

    pub fn set_location(&mut self, location: u16) {
        self.location = location;
    }

    pub fn add_data(&mut self, data: &mut Vec<u8>) {
        self.data.append(data);
    }

    pub fn get_data(&self) -> &Vec<u8> {
        &self.data
    }

    pub fn get_data_mut(&mut self) -> &mut Vec<u8> {
        &mut self.data
    }

    pub fn data_len(&self) -> usize {
        self.data.len()
    }
}

impl Default for Var {
    fn default() -> Self {
        Self {
            data_type: Type::None,
            data: vec![],
            location: 0
        }
    }
}
