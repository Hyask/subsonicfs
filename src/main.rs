extern crate env_logger;
extern crate fuse;
extern crate libc;
extern crate time;

use std::env;
use std::ffi::OsStr;
use libc::ENOENT;
use time::Timespec;
use fuse::{FileType, FileAttr, Filesystem, Request, ReplyData, ReplyEntry, ReplyAttr, ReplyDirectory};

extern crate sunk;

const SONG_ID: u64 = 1 << 63;
const ALBUM_ID: u64 = 1 << 62;
const ARTIST_ID: u64 = 1 << 61;


const TTL: Timespec = Timespec { sec: 1, nsec: 0 };                     // 1 second

const CREATE_TIME: Timespec = Timespec { sec: 1381237736, nsec: 0 };    // 2013-10-08 08:56

struct Artist {
    pub name: String,
    pub id: usize,
}

impl Artist {
    fn new_from_id(client: & sunk::Client, id: usize) -> Artist {
        let a = sunk::Artist::get(&client, id).unwrap();
        Artist {
            id,
            name: a.name,
        }
    }

    fn new_from_ino(client: & sunk::Client, ino: u64) -> Artist {
        let id = (ino & !ARTIST_ID) as usize;
        Artist::new_from_id(&client, id)
    }

    fn get_ino(&self) -> u64 {
        self.id as u64 | ARTIST_ID
    }

    fn get_attr(&self) -> FileAttr {
        FileAttr {
            ino: self.get_ino(),
            size: 0,
            blocks: 0,
            atime: CREATE_TIME,
            mtime: CREATE_TIME,
            ctime: CREATE_TIME,
            crtime: CREATE_TIME,
            kind: FileType::Directory,
            perm: 0o755,
            nlink: 2,
            uid: 501,
            gid: 20,
            rdev: 0,
            flags: 0,
        }
    }
}

fn get_dir_attr(ino: u64) -> FileAttr {
    FileAttr {
        ino: ino,
        size: 0,
        blocks: 0,
        atime: CREATE_TIME,
        mtime: CREATE_TIME,
        ctime: CREATE_TIME,
        crtime: CREATE_TIME,
        kind: FileType::Directory,
        perm: 0o755,
        nlink: 2,
        uid: 501,
        gid: 20,
        rdev: 0,
        flags: 0,
    }
}

const SUBFS_DIR_ATTR: FileAttr = FileAttr {
    ino: 1,
    size: 0,
    blocks: 0,
    atime: CREATE_TIME,
    mtime: CREATE_TIME,
    ctime: CREATE_TIME,
    crtime: CREATE_TIME,
    kind: FileType::Directory,
    perm: 0o755,
    nlink: 2,
    uid: 501,
    gid: 20,
    rdev: 0,
    flags: 0,
};

const HELLO_TXT_CONTENT: &'static str = "Hello World!\n";

const SUBFS_TXT_ATTR: FileAttr = FileAttr {
    ino: 2,
    size: 13,
    blocks: 1,
    atime: CREATE_TIME,
    mtime: CREATE_TIME,
    ctime: CREATE_TIME,
    crtime: CREATE_TIME,
    kind: FileType::RegularFile,
    perm: 0o644,
    nlink: 1,
    uid: 501,
    gid: 20,
    rdev: 0,
    flags: 0,
};

struct SubsonicFS<'subfs> {
    pub name: &'subfs str,
    pub client: sunk::Client,
}

impl<'subfs> SubsonicFS<'subfs> {
    fn get_artist_by_ino(&self, ino: u64) -> Artist {
        Artist::new_from_ino(&self.client, ino)
    }

    fn get_artist_by_id(&self, id: usize) -> Artist {
        Artist::new_from_id(&self.client, id)
    }

    fn get_artist_list(&self) -> Vec<Artist> {
        vec![
            Artist::new_from_id(&self.client, 1), // Lordi
        ]
    }

    fn get_artist_by_name(&self, name: &str) -> Option<Artist> {
        if name == "Lordi" {
            return Some(Artist::new_from_id(&self.client, 1));
        }
        return None;
    }
}

impl<'subfs> Filesystem for SubsonicFS<'subfs> {
    fn lookup(&mut self, _req: &Request, parent: u64, name: &OsStr, reply: ReplyEntry) {
        if parent == 1 {
            match name.to_str() {
                Some("hello.txt") => reply.entry(&TTL, &SUBFS_TXT_ATTR, 0),
                Some("Artists") => reply.entry(&TTL, &get_dir_attr(ARTIST_ID), 0),
                Some("Albums") => reply.entry(&TTL, &get_dir_attr(ALBUM_ID), 0),
                _ => reply.error(ENOENT),
            }
        } else if parent == ARTIST_ID {
            let a = self.get_artist_by_name(&name.to_str().unwrap());
            match a {
                Some(artist) => reply.entry(&TTL, &artist.get_attr(), 0),
                _ => reply.error(ENOENT),
            }
        } else {
            reply.error(ENOENT);
        }
    }

    fn getattr(&mut self, _req: &Request, ino: u64, reply: ReplyAttr) {
        if ino == 1 { reply.attr(&TTL, &SUBFS_DIR_ATTR) }
        else if ino == 2 { reply.attr(&TTL, &SUBFS_TXT_ATTR) }
        else if ino == ARTIST_ID { reply.attr(&TTL, &get_dir_attr(ARTIST_ID)) }
        else if ino == ALBUM_ID { reply.attr(&TTL, &SUBFS_DIR_ATTR) }
        else if ino == SONG_ID { reply.attr(&TTL, &SUBFS_DIR_ATTR) }
        else if ino & ARTIST_ID == ARTIST_ID { reply.attr(&TTL, &get_dir_attr(ino)) }
        else { reply.error(ENOENT) };
    }

    fn read(&mut self, _req: &Request, ino: u64, _fh: u64, offset: i64, _size: u32, reply: ReplyData) {
        if ino == 2 {
            reply.data(&HELLO_TXT_CONTENT.as_bytes()[offset as usize..]);
        } else {
            reply.error(ENOENT);
        }
    }

    fn readdir(&mut self, _req: &Request, ino: u64, _fh: u64, offset: i64, mut reply: ReplyDirectory) {
        let artists = &self.get_artist_list();
        let mut entries;
        match ino {
            1 => {
                entries = vec![
                    (1, FileType::Directory, "."),
                    (1, FileType::Directory, ".."),
                    (ARTIST_ID, FileType::Directory, "Artists"),
                    (ALBUM_ID, FileType::Directory, "Albums"),
                    (SONG_ID, FileType::Directory, "Songs"),
                    (2, FileType::RegularFile, "hello.txt"),
                ];
            }
            ARTIST_ID => {
                entries = vec![
                    (ARTIST_ID, FileType::Directory, "."),
                    (ARTIST_ID, FileType::Directory, ".."),
                ];
                for a in artists {
                    entries.push((a.get_ino(), FileType::Directory, &a.name))
                }
            }
            _ => entries = vec![],
        }


        // Offset of 0 means no offset.
        // Non-zero offset means the passed offset has already been seen, and we should start after
        // it.
        let to_skip = if offset == 0 { offset } else { offset + 1 } as usize;
        for (i, entry) in entries.into_iter().enumerate().skip(to_skip) {
            reply.add(entry.0, i as i64, entry.1, entry.2);
        }
        reply.ok();
    }
}

fn main() {
    env_logger::init();
    let mountpoint = env::args_os().nth(1).unwrap();
    let options = ["-o", "ro", "-o", "fsname=subsonicfs"]
        .iter()
        .map(|o| o.as_ref())
        .collect::<Vec<&OsStr>>();

    let site = "http://localhost/";
    let username = "skia";
    let password = "skia";

    let client = sunk::Client::new(site, username, password).unwrap();

    let fs = SubsonicFS {
        name: "Subsonic FS",
        client: client,
    };

    // println!("client: {:#?}", client);
    // let an_artist = sunk::Artist::get(&client, 1);
    // // let artist_info = an_artist.info(&client);
    // // let artists_albums = an_artist.albums(&client);
    // println!("artist: {:#?}", an_artist);
    // // println!("artist_info: {:#?}", artist_info);
    // // println!("artists_albums: {:#?}", artists_albums);

    // let an_album = sunk::Album::get(&client, 1);
    // println!("album: {:#?}", an_album);

    // let a_song = sunk::song::Song::get(&client, 1);
    // println!("song: {:#?}", a_song);

    // println!("ARTIST_ID: {}", ARTIST_ID);


    fuse::mount(fs, &mountpoint, &options).unwrap();
}
