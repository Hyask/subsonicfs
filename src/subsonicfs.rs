
extern crate fuse;

use libc::{ENOENT,EOF};
use time::Timespec;
use std::collections::HashMap;
use fuse::{FileType, FileAttr, Filesystem, Request, ReplyData, ReplyEntry, ReplyAttr, ReplyDirectory};
use sunk::{Artist, Streamable, ListType, Album};
use std::ffi::OsStr;

// trait SubFSFile {
//     type File;
//     fn get_ino_from_id(id: usize) -> u64;
//     fn get_id_from_ino(ino: u64) -> usize;
//     fn new_from_id(client: & sunk::Client, id: usize) -> Self::File;
//     fn get_ino(&self) -> u64;
//     fn get_attr(&self) -> FileAttr;
// }

const ARTIST_ID: u64 = 1 << 31;
const ALBUM_ID: u64 = 1 << 32;

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
    uid: 65534, // nobody
    gid: 65534, // nobody
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
    uid: 65534, // nobody
    gid: 65534, // nobody
    rdev: 0,
    flags: 0,
};


pub struct SubsonicFS<'subfs> {
    pub name: &'subfs str,
    pub client: sunk::Client,
    pub artists: Vec<Artist>,
    pub artists_name_to_index: HashMap<String, usize>, // key is the name
    pub albums: Vec<Album>,
    pub albums_name_to_index: HashMap<String, usize>, // key is the name
    pub artist_ino_to_album_ino_list: HashMap<u64, Vec<u64>>,
}

impl<'subfs> SubsonicFS<'subfs> {
    pub fn new(name: &str, client: sunk::Client) -> SubsonicFS {
        let mut s = SubsonicFS {
            name: name,
            client: client,
            artists: Vec::new(),
            artists_name_to_index: HashMap::new(),
            albums: Vec::new(),
            albums_name_to_index: HashMap::new(),
            artist_ino_to_album_ino_list: HashMap::new(),
        };
        s.build_artist_list();
        s
    }

    fn add_new_artist(&mut self, mut artist: Artist) {
        artist.name = artist.name.replace("/", "-");
        self.artists.push(artist);
        let artist = &self.artists.last().unwrap();
        let name = artist.name.clone();
        self.artists_name_to_index.insert(name, self.artists.len() - 1);
        // TODO
        // self.build_album_list(artist);
    }

    fn add_new_album(&mut self, album: Album) {
        self.albums.push(album);
        let album = &self.albums.last().unwrap();
        let name = album.name.clone();
        self.albums_name_to_index.insert(name, self.albums.len() - 1);
        // let artist_name = &album.artist.unwrap();
        // match self.get_artist_by_name(artist_name) {
        //     Some(artist) => {
        //         let artist_ino = self.get_artist_ino(artist);
        //         // self.artist_ino_to_album_ino_list.insert(artist_ino, )
        //     },
        //     None => println!("Oops, no artist found for this album: {:#?}", album)
        // }
    }

    fn build_artist_list(&mut self) {
        let artist_list = sunk::Artist::list(&self.client, sunk::search::ALL).unwrap();
        for a in artist_list {
            self.add_new_artist(a);
        }
    }

    fn build_album_list(&mut self, artist: &Artist) {
        let album_list = artist.albums(&self.client).unwrap();
        for a in album_list {
            self.add_new_album(a);
        }
    }

    fn get_albums_for_artist(&self, artist: &Artist) -> Option<Vec<Album>> {
        Some(artist.albums(&self.client).unwrap())
    }

    fn get_artist_list(&self) -> & Vec<Artist> {
        &self.artists
    }

    fn get_album_list(&self) -> & Vec<Album> {
        &self.albums
    }

    fn get_artist_by_name(&self, name: &str) -> Option<&Artist> {
        println!("name: {}", name);
        match self.artists_name_to_index.get(name) {
            Some(&index) => self.artists.get(index),
            None => None,
        }
    }

    fn get_artist_by_ino(&self, ino: u64) -> Option<&Artist> {
        println!("ino: {}", ino);
        let index = ino - ARTIST_ID - 1;
        self.artists.get(index as usize)
    }

    fn get_artist_index(&self, artist: &Artist) -> usize {
        *self.artists_name_to_index.get(&artist.name).unwrap()
    }

    fn get_album_index(&self, album: &Album) -> usize {
        *self.albums_name_to_index.get(&album.name).unwrap()
    }

    pub fn get_artist_ino(&self, artist: &Artist) -> u64 {
        let ino = ARTIST_ID + 1 + self.get_artist_index(artist) as u64;
        ino
    }

    pub fn get_album_ino(&self, album: &Album) -> u64 {
        let ino = ALBUM_ID + 1 + self.get_album_index(album) as u64;
        ino
    }

    pub fn get_artist_attr(&self, artist: &Artist) -> FileAttr {
        FileAttr {
            ino: self.get_artist_ino(&artist),
            size: artist.album_count as u64,
            blocks: 0,
            atime: CREATE_TIME,
            mtime: CREATE_TIME,
            ctime: CREATE_TIME,
            crtime: CREATE_TIME,
            kind: FileType::Directory,
            perm: 0o755,
            nlink: 2,
            uid: 65534, // nobody
            gid: 65534, // nobody
            rdev: 0,
            flags: 0,
        }
    }

    fn get_dir_attr(&self, ino: u64) -> FileAttr {
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
            uid: 65534, // nobody
            gid: 65534, // nobody
            rdev: 0,
            flags: 0,
        }
    }
}

impl<'subfs> Filesystem for SubsonicFS<'subfs> {
    fn lookup(&mut self, _req: &Request, parent: u64, name: &OsStr, reply: ReplyEntry) {
        println!("lookup");
        if parent == 1 {
            match name.to_str() {
                Some("hello.txt") => reply.entry(&TTL, &SUBFS_TXT_ATTR, 0),
                Some("Artists") => reply.entry(&TTL, &self.get_dir_attr(ARTIST_ID), 0),
                _ => reply.error(ENOENT),
            }
        } else if parent == ARTIST_ID {
            let a = self.get_artist_by_name(&name.to_str().unwrap());
            println!("PLOP: {:?}", a);
            match a {
                Some(artist) => reply.entry(&TTL, &self.get_artist_attr(&artist), 0),
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
        println!("readdir: {}", ino);
        let mut entries;
        match ino {
            1 => {
                entries = vec![
                    (1, FileType::Directory, "."),
                    (1, FileType::Directory, ".."),
                    (2, FileType::RegularFile, "hello.txt"),
                    (ARTIST_ID, FileType::RegularFile, "Artists"),
                ];
            }
            ARTIST_ID => {
                entries = vec![
                    (ARTIST_ID, FileType::Directory, "."),
                    (1, FileType::Directory, ".."),
                ];
                for i in 0..self.get_artist_list().len() {
                    let a = &self.get_artist_list()[i];
                    entries.push((self.get_artist_ino(&a), FileType::Directory, &a.name));
                }
                // println!("entries: {:?}", entries);
            }
            _ => {
                if (ino & ARTIST_ID) == ARTIST_ID { // This is an artist folder
                    entries = vec![
                        (ino, FileType::Directory, "."),
                        (ARTIST_ID, FileType::Directory, ".."),
                    ];
                    let artist = &self.get_artist_by_ino(ino).unwrap();
                    let albums = &self.get_albums_for_artist(&artist).unwrap();
                    for al in albums {
                        println!("album: {:?}", al);
                        let i = self.get_album_index(&al);
                        let a = &self.get_album_list()[i];

                        entries.push((self.get_album_ino(&a), FileType::Directory, &a.name));
                    }
                } else {
                    entries = vec![];
                }
            }
        }


        // Offset of 0 means no offset.
        // Non-zero offset means the passed offset has already been seen, and we should start after
        // it.
        let to_skip = if offset == 0 { offset } else { offset + 1 } as usize;
        for (i, entry) in entries.into_iter().enumerate().skip(to_skip) {
            // println!("entry: {:?}", entry);
            reply.add(entry.0, i as i64, entry.1, entry.2);
        }
        reply.ok();
    }
}
