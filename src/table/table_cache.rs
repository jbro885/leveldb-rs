use std::collections::HashMap;
use filename;
use super::table::Table;

pub struct TableCache<V> {
    cache: HashMap<u64, V>, // TODO: use more smart cache
    db_name: String,
}

impl<V> TableCache<V> {
    pub fn new(name: &str) -> Self {
        Self {
            cache: HashMap::new(),
            db_name: name.to_owned(),
        }
    }

    pub fn get(&mut self, file_number: u64, size: u64) -> &V {
        if let Some(v) = self.cache.get(&file_number) {
            v
        } else {
            let name = filename::FileType::Table(&self.db_name, file_number).filename();
            Table::open(&name, size);

            unreachable!()
        }
    }
}