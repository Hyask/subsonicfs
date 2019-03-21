extern crate sunk;
extern crate fuse;

use fuse::{FileType, FileAttr, Filesystem, Request, ReplyData, ReplyEntry, ReplyAttr, ReplyDirectory};

use time::Timespec;

type Artist = sunk::Artist;


const CREATE_TIME: Timespec = Timespec { sec: 1381237736, nsec: 0 };    // 2013-10-08 08:56

//pub struct Artist {
    //pub name: String,
    //pub id: String,
//}

impl Artist {
    pub fn new_from_id(client: & sunk::Client, id: &String) -> Artist {
        let a = sunk::Artist::get(&client, id).unwrap();
        Artist {
            id: a.id,
            name: a.name,
        }
    }

    [>
    fn new_from_ino(client: & sunk::Client, ino: u64) -> Artist {
        let id = (ino & !ARTIST_ID) as usize;
        Artist::new_from_id(&client, id)
    }
    */

    pub fn get_ino(&self) -> u64 {
        12
    }

    pub fn get_attr(&self) -> FileAttr {
        FileAttr {
            ino: self.get_ino(),
            size: 0,
            blocks: 0,
            atime: CREATE_TIME,
            mtime: CREATE_TIME,
            ctime: CREATE_TIME,
            crtime: CREATE_TIME,
            kind: FileType::RegularFile,
            perm: 0o755,
            nlink: 2,
            uid: 501,
            gid: 20,
            rdev: 0,
            flags: 0,
        }
    }
}

