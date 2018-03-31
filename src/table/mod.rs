mod builder;
mod table_writer;
mod block_builder;
mod block_format;

use version::{FileMetaData, FileMetaDataBuilder};
use memdb::MemDBIterator;
use filename;
use self::builder::TableBuilder;
use slice::Slice;
// use self::block_format;

enum Compression {
    No,
}

pub fn bulid(
    dbname: &str,
    iterator: &mut MemDBIterator,
    num: u64,
) -> Result<FileMetaData, &'static str> {
    let mut meta_builder = FileMetaDataBuilder::new();
    meta_builder.file_num(num);

    let fname = filename::FileType::Table(dbname, num).filename();
    let mut builder = TableBuilder::new(&fname);

    for (i, (k, v)) in iterator.enumerate() {
        if i == 0 {
            meta_builder.smallest(k.clone());
        }

        meta_builder.largest(k.clone()); // must clone

        let mut s = Slice::from(&k);
        let v = Slice::from(&v);
        builder.add(&mut s, &v);
    }

    builder.build();

    meta_builder.file_size(builder.size() as u64);
    meta_builder.build()
}
