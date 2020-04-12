
extern crate fuse;
extern crate sunk;

use std::rc::Rc;
use std::cell::RefCell;
use std::borrow::BorrowMut;
use libc::{ENOENT,EOF};
use time::Timespec;
use std::collections::HashMap;
use fuse::{FileType, FileAttr, Filesystem, Request, ReplyData, ReplyEntry, ReplyAttr, ReplyDirectory};
use sunk::Streamable;
use std::ffi::OsStr;

const ARTIST_ID: u64 = 1 << 31;
const ALBUM_ID: u64 = 1 << 32;
const SONG_ID: u64 = 1 << 33;

const TTL: Timespec = Timespec { sec: 100, nsec: 0 };                     // 100 seconds

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

// #[derive(Debug, Clone)]
pub struct Artist {
    pub name: String,
    pub sonic_artist: sunk::Artist,
    pub albums: RefCell<Vec<Rc<Album>>>,
}

impl Artist {
}

#[derive(Debug)]
pub struct Album {
    pub name: String,
    pub sonic_album: sunk::Album,
    pub songs: Vec<Rc<Song>>,
}

#[derive(Debug)]
pub struct Song {
    pub name: String,
    pub sonic_song: sunk::song::Song,
}

pub struct SubsonicFS<'subfs> {
    pub name: &'subfs str,
    pub client: sunk::Client,
    pub artists: Vec<Rc<Artist>>,
    pub artists_name_to_index: HashMap<String, usize>, // key is the name
    pub albums: Vec<Rc<Album>>,
    pub albums_name_to_index: HashMap<String, usize>, // key is the name
    pub songs: Vec<Rc<Song>>,
    pub songs_name_to_index: HashMap<String, usize>, // key is the name
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
            songs: Vec::new(),
            songs_name_to_index: HashMap::new(),
        };
        s.build_artist_list();
        s
    }

    fn add_new_artist(&mut self, mut artist: Artist) {
        println!("Artist: {:#?}", artist.name);
        // self.build_album_list(&mut artist);
        artist.name = artist.name.replace("/", "-");
        self.artists.push(Rc::new(artist));
        let artist = &self.artists.last().unwrap();
        let name = artist.name.clone();
        self.artists_name_to_index.insert(name, self.artists.len() - 1);
    }

    fn add_new_album(&mut self, mut album: Album) -> Rc<Album> {
        println!("Album: {:#?}", album.name);
        self.build_song_list(&mut album);
        self.albums.push(Rc::new(album));
        let album = &self.albums.last().unwrap();
        let name = album.name.clone();
        self.albums_name_to_index.insert(name, self.albums.len() - 1);
        self.albums.last().unwrap().clone()
    }

    fn add_new_song(&mut self, song: Song) -> Rc<Song> {
        self.songs.push(Rc::new(song));
        let song = &self.songs.last().unwrap();
        let name = song.name.clone();
        self.songs_name_to_index.insert(name, self.songs.len() - 1);
        self.songs.last().unwrap().clone()
    }

    fn build_artist_list(&mut self) {
        let artist_list = sunk::Artist::list(&self.client, sunk::search::ALL).unwrap();
        for a in artist_list {
            let artist = Artist {
                name: a.name.clone(),
                sonic_artist: a,
                albums: RefCell::new(Vec::new()),
            };
            self.add_new_artist(artist);
        }
    }

    fn build_album_list(&mut self, artist: & Artist) {
        let album_list = artist.sonic_artist.albums(&self.client).unwrap();
        let mut albums = Vec::new();
        for a in album_list {
            let album = Album {
                name: a.name.clone(),
                sonic_album: a,
                songs: Vec::new(),
            };
            albums.push(self.add_new_album(album));
        }
        let mut artist_albums = artist.albums.borrow_mut();
        *artist_albums = albums;
    }

    fn build_song_list(&mut self, album: &mut Album) {
        let song_list = album.sonic_album.songs(&self.client).unwrap();
        for mut s in song_list {
            // s.set_max_bit_rate(128);
            let song = Song {
                name: s.title.clone(),
                sonic_song: s,
            };
            album.songs.push(self.add_new_song(song));
        }
    }

    fn get_artist_by_name(&self, name: &str) -> Option<&Rc<Artist>> {
        println!("artist name: {}", name);
        match self.artists_name_to_index.get(name) {
            Some(&index) => self.artists.get(index),
            None => None,
        }
    }

    fn get_album_by_name(&self, name: &str) -> Option<&Rc<Album>> {
        println!("album name: {}", name);
        match self.albums_name_to_index.get(name) {
            Some(&index) => self.albums.get(index),
            None => None,
        }
    }

    fn get_song_by_name(&self, name: &str) -> Option<&Rc<Song>> {
        println!("album name: {}", name);
        match self.songs_name_to_index.get(name) {
            Some(&index) => self.songs.get(index),
            None => None,
        }
    }

    fn get_artist_albums(&mut self, artist: &Artist) -> Vec<Rc<Album>> {
        println!("get_artist_albums: {}", artist.name);
        if artist.albums.clone().into_inner().len() == 0 {
            self.build_album_list(artist);
        }
        artist.albums.clone().into_inner()
    }

    fn get_artist_by_ino(&self, ino: u64) -> Option<&Rc<Artist>> {
        println!("ino: {}", ino);
        let index = ino - ARTIST_ID - 1;
        self.artists.get(index as usize)
    }

    fn get_album_by_ino(&self, ino: u64) -> Option<&Rc<Album>> {
        println!("ino: {}", ino);
        let index = ino - ALBUM_ID - 1;
        self.albums.get(index as usize)
    }

    fn get_song_by_ino(&self, ino: u64) -> Option<&Rc<Song>> {
        println!("ino: {}", ino);
        let index = ino - SONG_ID - 1;
        self.songs.get(index as usize)
    }

    fn get_artist_index(&self, artist: &Artist) -> usize {
        *self.artists_name_to_index.get(&artist.name).unwrap()
    }

    fn get_album_index(&self, album: &Album) -> usize {
        *self.albums_name_to_index.get(&album.name).unwrap()
    }

    fn get_song_index(&self, song: &Song) -> usize {
        *self.songs_name_to_index.get(&song.name).unwrap()
    }

    pub fn get_artist_ino(&self, artist: &Artist) -> u64 {
        let ino = ARTIST_ID + 1 + self.get_artist_index(artist) as u64;
        ino
    }

    pub fn get_album_ino(&self, album: &Album) -> u64 {
        let ino = ALBUM_ID + 1 + self.get_album_index(album) as u64;
        ino
    }

    pub fn get_song_ino(&self, song: &Song) -> u64 {
        let ino = SONG_ID + 1 + self.get_song_index(song) as u64;
        ino
    }

    pub fn get_artist_attr(&self, artist: &Artist) -> FileAttr {
        FileAttr {
            ino: self.get_artist_ino(&artist),
            size: artist.sonic_artist.album_count as u64,
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

    pub fn get_album_attr(&self, album: &Album) -> FileAttr {
        FileAttr {
            ino: self.get_album_ino(&album),
            size: album.sonic_album.song_count,
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

    pub fn get_song_attr(&self, song: &Song) -> FileAttr {
        FileAttr {
            ino: self.get_song_ino(&song),
            size: song.sonic_song.size,
            blocks: 0,
            atime: CREATE_TIME,
            mtime: CREATE_TIME,
            ctime: CREATE_TIME,
            crtime: CREATE_TIME,
            kind: FileType::RegularFile,
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
        println!("lookup: {:?}", name);
        if parent == 1 {
            match name.to_str() {
                Some("hello.txt") => reply.entry(&TTL, &SUBFS_TXT_ATTR, 0),
                Some("Artists") => reply.entry(&TTL, &self.get_dir_attr(ARTIST_ID), 0),
                _ => reply.error(ENOENT),
            }
        } else if parent == ARTIST_ID {
            let ar = self.get_artist_by_name(&name.to_str().unwrap());
            match ar {
                Some(artist) => reply.entry(&TTL, &self.get_artist_attr(&artist), 0),
                _ => reply.error(ENOENT),
            }
        } else if (parent & ARTIST_ID) == ARTIST_ID { // This is an album folder
            let al = self.get_album_by_name(&name.to_str().unwrap());
            match al {
                Some(album) => reply.entry(&TTL, &self.get_album_attr(&album), 0),
                _ => reply.error(ENOENT),
            }
        } else if (parent & ALBUM_ID) == ALBUM_ID { // This is a song file
            let s = self.get_song_by_name(&name.to_str().unwrap());
            match s {
                Some(song) => reply.entry(&TTL, &self.get_song_attr(&song), 0),
                _ => reply.error(ENOENT),
            }
        } else {
            reply.error(ENOENT);
        }
    }

    fn getattr(&mut self, _req: &Request, ino: u64, reply: ReplyAttr) {
        println!("getattr: {}", ino);
        if ino == 1 { reply.attr(&TTL, &SUBFS_DIR_ATTR) }
        else if ino == 2 { reply.attr(&TTL, &SUBFS_TXT_ATTR) }
        else { reply.error(ENOENT) };
    }

    fn read(&mut self, _req: &Request, ino: u64, _fh: u64, offset: i64, _size: u32, reply: ReplyData) {
        println!("read - ino: {}, _fh: {}, offset: {}, _size: {}", ino, _fh, offset, _size);
        if ino == 2 {
            reply.data(&HELLO_TXT_CONTENT.as_bytes()[offset as usize..]);
        } else if (ino & SONG_ID) == SONG_ID { // This is a song
            let s = &self.get_song_by_ino(ino);
            println!("Song: {:#?}", s);
            match s {
                Some(song) => {

                    let size;
                    if offset as usize + _size as usize > song.sonic_song.size as usize {
                        size = song.sonic_song.size as usize - offset as usize;
                    } else {
                        size = _size as usize;
                    }
                    println!("offset: {}, size: {}, _size: {}, song.size: {}", offset, size, _size, song.sonic_song.size);
                    if offset as usize >= song.sonic_song.size as usize {
                        reply.error(EOF);
                    } else {
                        reply.data(&song.sonic_song.stream(&self.client).unwrap()[offset as usize..offset as usize + size as usize]);
                    }

                }
                None => reply.error(ENOENT),
            }

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
                for i in 0..self.artists.len() {
                    let a = &self.artists[i];
                    entries.push((self.get_artist_ino(&a), FileType::Directory, &a.name));
                }
            }
            _ => {
                if (ino & ARTIST_ID) == ARTIST_ID { // This is an artist folder
                    entries = vec![
                        (ino, FileType::Directory, "."),
                        (ARTIST_ID, FileType::Directory, ".."),
                    ];
                    let artist = &self.get_artist_by_ino(ino).unwrap().clone();
                    let albums = &self.get_artist_albums(artist);
                    for al in albums {
                        let i = self.get_album_index(&al);
                        let a = &self.albums[i];

                        entries.push((self.get_album_ino(&a), FileType::Directory, &a.name));
                    }
                } else if (ino & ALBUM_ID) == ALBUM_ID { // This is an album folder
                    entries = vec![
                        (ino, FileType::Directory, "."),
                        (ARTIST_ID, FileType::Directory, ".."),
                    ];
                    let album = &self.get_album_by_ino(ino).unwrap();
                    let songs = &album.songs;
                    for s in songs {
                        let i = self.get_song_index(&s);
                        let song = &self.songs[i];

                        entries.push((self.get_song_ino(&song), FileType::Directory, &song.name));
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
