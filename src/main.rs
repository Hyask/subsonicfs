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

    let fs = SubsonicFS::new("Subsonic FS", client);

    fuse::mount(fs, &mountpoint, &options).unwrap();
}
