use crate::id_table::IdTable;

pub struct World {
    id_table: IdTable,
}

impl World {
    pub fn new() -> Self {
        Self {
            id_table: IdTable::new(),
        }
    }
}
