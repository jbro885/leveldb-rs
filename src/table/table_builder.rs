use crc::{Hasher32, crc32};
use slice;
use slice::{ByteWrite, Bytes, BytesMut};
use std::fs;
use std::io;
use std::io::BufWriter;
use table::{Compression, block_builder::BlockBuilder, format::{BlockHandle, Footer}};

pub struct TableBuilder<T: io::Write> {
    writer: TableWriter<T>,
    data_block: BlockBuilder,
    index_block: BlockBuilder,
    filter_block: Option<u64>, // FIX
    pending_handle: BlockHandle,
    pending_index_entry: bool,
    last_key: Bytes,
}

pub const TRAILER_SIZE: usize = 5;

pub fn new(fname: &str) -> TableBuilder<BufWriter<fs::File>> {
    debug!("Open file {:?} for table", fname);
    let fd = fs::OpenOptions::new() // add read permission?
        .write(true)
        .create(true)
        .open(fname)
        .unwrap();

    TableBuilder::new(BufWriter::new(fd))
}

impl<T: io::Write> TableBuilder<T> {
    pub fn new(w: T) -> Self {
        Self {
            writer: TableWriter::new(w),
            data_block: BlockBuilder::new(),
            index_block: BlockBuilder::new(),
            pending_handle: BlockHandle::new(),
            pending_index_entry: false,
            filter_block: None,
            last_key: Bytes::new(),
        }
    }

    pub fn add(&mut self, key: &Bytes, value: &Bytes) {
        let mut k = key.clone();
        if self.pending_index_entry {
            slice::short_successor(k.to_mut());

            let content = self.pending_handle.encode();
            self.index_block.add(&k, &content);
            self.pending_index_entry = false;
        }

        self.data_block.add(&k, value);
        self.last_key = k;

        // FIX: 1024
        if self.data_block.estimated_current_size() >= 1024 {
            debug!("Estimated size exceeds specifed size");
            self.build()
        }
    }

    pub fn build(&mut self) {
        self.flush();

        if let Some(_) = self.filter_block {
            // TODO: write filter block
        }

        let metaindex_block_handle = {
            let mut meta_index_block = BlockBuilder::new();
            if let Some(_) = self.filter_block {
                // TODO: write filter block
            }
            let content = meta_index_block.build();
            debug!(
                "Write metaindex block handle offset={:?}, size={:?}",
                self.writer.offset(),
                content.len(),
            );
            self.write_block(&content)
        };

        // index
        let index_block_handle = {
            if self.pending_index_entry {
                slice::short_successor(self.last_key.to_mut());
                // let ss = TableBuilder::succ(&self.last_key);
                let content = self.pending_handle.encode();
                self.index_block.add(&self.last_key, &content);
                self.pending_index_entry = false;
            }
            let content = self.index_block.build();
            debug!(
                "Write index block handle offset={:?}, size={:?}",
                self.writer.offset(),
                content.len(),
            );
            self.write_block(&content)
        };

        // footer
        {
            let footer = Footer::new(index_block_handle, metaindex_block_handle);
            let content = footer.encode();
            debug!("Write footer to file. offset is {:?}", self.size());
            self.writer
                .write(content.as_ref())
                .expect("Writing data is failed");
        }
    }

    pub fn size(&self) -> usize {
        self.writer.offset() as usize
    }

    fn flush(&mut self) {
        if self.data_block.empty() {
            return;
        }

        let content = self.data_block.build();
        debug!(
            "Flush data offset={:?}, size={:?}",
            self.writer.offset(),
            content.len(),
        );
        self.pending_handle = self.write_block(&content);
        self.pending_index_entry = true;
    }

    fn write_block(&mut self, content: &Bytes) -> BlockHandle {
        let kind = Compression::No;
        self.write_raw_block(content, kind)
    }

    fn write_raw_block(&mut self, content: &Bytes, kindt: Compression) -> BlockHandle {
        // offset must be set before writer.write
        let bh = BlockHandle::from((content.len()) as u64, self.writer.offset());

        let kind = kindt as u8;
        let content_slice = content.as_ref();
        self.writer
            .write(content_slice)
            .expect("Writing data is failed");

        // crc
        {
            let crc = {
                let mut digest = crc32::Digest::new(crc32::IEEE);
                digest.write(content_slice);
                digest.write(&[kind]);
                digest.sum32()
            };

            let mut trailer = BytesMut::with_capacity(TRAILER_SIZE);
            trailer.write_u8(kind);
            trailer.write_u32(crc);
            self.writer
                .write(trailer.as_ref())
                .expect("Writing data is failed");
        }

        bh
    }
}

pub struct TableWriter<T> {
    inner: T,
    offset: usize,
}

impl<T: io::Write> TableWriter<T> {
    pub fn new(writer: T) -> TableWriter<T> {
        TableWriter {
            inner: writer,
            offset: 0,
        }
    }

    pub fn write(&mut self, content: &[u8]) -> Result<usize, io::Error> {
        debug!("write data to table {:?}", content);
        self.offset += content.len();
        self.inner.write(content)
    }

    pub fn offset(&self) -> u64 {
        self.offset as u64
    }
}
