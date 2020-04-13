mod subsonicfs;

extern crate env_logger;
extern crate fuse;
extern crate libc;
extern crate time;
extern crate sunk;

use std::env;
use std::ffi::OsStr;
use subsonicfs::SubsonicFS;

use std::path::PathBuf;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(name = "subsonicfs", about = "A Subsonic filesystem using FUSE.")]
struct Opt {
    #[structopt(short, long, default_value = "http://demo.subsonic.org")]
    server: String,

    #[structopt(short, long, default_value = "guest2")]
    username: String,

    #[structopt(short, long, default_value = "guest")]
    password: String,

    #[structopt(parse(from_os_str))]
    mount_point: PathBuf,
}

fn main() {
    env_logger::init();
    let opt = Opt::from_args();

    let fuse_options = ["-o", "ro", "-o", "fsname=subsonicfs"]
        .iter()
        .map(|o| o.as_ref())
        .collect::<Vec<&OsStr>>();

    let client = sunk::Client::new(&opt.server, &opt.username, &opt.password).unwrap();

    let fs = SubsonicFS::new("Subsonic FS", client);

    fuse::mount(fs, &opt.mount_point, &fuse_options).unwrap();
}
