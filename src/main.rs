mod subsonicfs;

extern crate env_logger;
extern crate fuse;
extern crate libc;
extern crate time;
extern crate sunk;

use std::env;
use std::ffi::OsStr;
use subsonicfs::SubsonicFS;




fn main() {
    env_logger::init();
    let mountpoint = env::args_os().nth(1).unwrap();
    let options = ["-o", "ro", "-o", "fsname=subsonicfs"]
        .iter()
        .map(|o| o.as_ref())
        .collect::<Vec<&OsStr>>();

    let site = "https://festival.libskia.so";
    let username = "skia";
    let password = "plop4000";

    let client = sunk::Client::new(site, username, password).unwrap();
    //let song = Song::new_from_id(&client, 1);

    let mut fs = SubsonicFS::new("Subsonic FS", client);

    // fs.build_artist_list();
    // println!("{:?}", fs.get_artist_list());
    // println!("{:?}", fs.get_artist_list().len());

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
