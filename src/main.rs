extern crate env_logger;
extern crate fuse;
extern crate libc;
extern crate time;

use std::env;
use std::ffi::OsStr;
use libc::{ENOENT,EOF};
use time::Timespec;
use fuse::{FileType, FileAttr, Filesystem, Request, ReplyData, ReplyEntry, ReplyAttr, ReplyDirectory};
use std::collections::HashMap;

extern crate sunk;
use sunk::{Artist, Streamable, ListType};

const TTL: Timespec = Timespec { sec: 1, nsec: 0 };                     // 1 second

const CREATE_TIME: Timespec = Timespec { sec: 1381237736, nsec: 0 };    // 2013-10-08 08:56
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
    pub artists: Vec<Artist>,
    pub artists_by_name: HashMap<&'subfs str, &'subfs Artist>, // key is the name
    pub artists_by_ino: HashMap<u64, &'subfs Artist>, // key is the inode
}

impl<'subfs> SubsonicFS<'subfs> {
    fn new(name: &str, client: sunk::Client) -> SubsonicFS {
        SubsonicFS {
            name: name,
            client: client,
            artists: Vec::new(),
            artists_by_name: HashMap::new(),
            artists_by_ino: HashMap::new(),
        }
    }

    fn add_new_artist(&mut self, artist: Artist) {

    }

    fn build_artist_list(&mut self) {
        let artist_list = sunk::Artist::list(&self.client, sunk::search::ALL).unwrap();
        print!("{:#?}", artist_list);
        for a in artist_list {
            self.artists.push(a);
        }
        print!("{:#?}", self.artists);
    }

    fn get_artist_list(&self) -> & Vec<Artist> {
        &self.artists
        //if self.artists.len() < 1 {
            //self.artists = vec![
                //Artist::new_from_id(&self.client, &String::from("AR1")), // Lordi
            //];
            //self.artists
        //} else {
            //self.artists
        //}
    }
}

impl<'subfs> Filesystem for SubsonicFS<'subfs> {
    fn lookup(&mut self, _req: &Request, parent: u64, name: &OsStr, reply: ReplyEntry) {
        println!("lookup");
        if parent == 1 {
            match name.to_str() {
                Some("hello.txt") => reply.entry(&TTL, &SUBFS_TXT_ATTR, 0),
                _ => reply.error(ENOENT),
            }
        } else {
            reply.error(ENOENT);
        }
    }

    fn getattr(&mut self, _req: &Request, ino: u64, reply: ReplyAttr) {
        println!("getattr");
        if ino == 1 { reply.attr(&TTL, &SUBFS_DIR_ATTR) }
        else if ino == 2 { reply.attr(&TTL, &SUBFS_TXT_ATTR) }
        else { reply.error(ENOENT) };
    }

    fn read(&mut self, _req: &Request, ino: u64, _fh: u64, offset: i64, _size: u32, reply: ReplyData) {
        println!("read");
        if ino == 2 {
            reply.data(&HELLO_TXT_CONTENT.as_bytes()[offset as usize..]);
        } else {
            reply.error(ENOENT);
        }
    }

    fn readdir(&mut self, _req: &Request, ino: u64, _fh: u64, offset: i64, mut reply: ReplyDirectory) {
        println!("readdir");
        let mut entries;
        match ino {
            1 => {
                entries = vec![
                    (1, FileType::Directory, "."),
                    (1, FileType::Directory, ".."),
                    (2, FileType::RegularFile, "hello.txt"),
                ];
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

    let site = "http://127.0.0.1:5000/";
    let username = "skia";
    let password = "skia";
    // let site = "http://demo.subsonic.org/";
    // let username = "guest4";
    // let password = "guest";

    let client = sunk::Client::new(site, username, password).unwrap();
    //let song = Song::new_from_id(&client, 1);

    let mut fs = SubsonicFS::new("Subsonic FS", client);

    fs.build_artist_list();

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
