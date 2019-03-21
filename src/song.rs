extern crate sunk;
type Song = sunk::song::Song;

trait SubFSFile {
    type File;
    fn get_ino_from_id(id: usize) -> u64;
    fn get_id_from_ino(ino: u64) -> usize;
    fn new_from_id(client: & sunk::Client, id: usize) -> Self::File;
    fn new_from_ino(client: & sunk::Client, ino: u64) -> Self::File;
    fn get_ino(&self) -> u64;
    fn get_attr(&self) -> FileAttr;
}

impl SubFSFile for Song {
    type File = Song;

    fn get_ino_from_id(id: usize) -> u64 {
        id as u64 | SONG_ID
    }
    fn get_id_from_ino(ino: u64) -> usize {
        ino as usize & !SONG_ID as usize
    }

    fn new_from_id(client: & sunk::Client, id: usize) -> Song {
        Song::get(&client, id as u64).unwrap()
    }

    fn new_from_ino(client: & sunk::Client, ino: u64) -> Song {
        let id = Song::get_id_from_ino(ino);
        Song::new_from_id(&client, id)
    }

    fn get_ino(&self) -> u64 {
        Song::get_ino_from_id(self.id as usize)
    }

    fn get_attr(&self) -> FileAttr {
        FileAttr {
            ino: self.get_ino(),
            size: self.size,
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

