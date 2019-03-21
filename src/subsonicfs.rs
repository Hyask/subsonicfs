extern crate fuse;

use fuse::{File, FileAttr};

trait SubFSFile {
    type File;
    fn get_ino_from_id(id: usize) -> u64;
    fn get_id_from_ino(ino: u64) -> usize;
    fn new_from_id(client: & sunk::Client, id: usize) -> Self::File;
    fn get_ino(&self) -> u64;
    fn get_attr(&self) -> FileAttr;
}


