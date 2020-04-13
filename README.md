# SubsonicFS


This is a FUSE filesystem providing access to a subsonic server. This allows
using most standard music player with most standard subsonic server, which is
quite convenient if the web UI sucks or if you want to bind global shortcut for
"Play/Pause" for example.

## How to use

There are no pre-built binary, but since it's developed in Rust, it's no big
deal to build it yourself (as long as you have 128, 12GHz cores available for all
the dependencies!).


Here are the rough steps to use it:

```bash
git clone https://git.libskia.so/skia/subsonicfs
cd subsonicfs
cargo run \
    --server https://your.subsonic.serv.er \
    --username subsonic_user \
    --password subsonic_password \
    /path/to/your/mount_point
```

You will then find your library in the given mount point, and any standard
desktop player should be able to access it and read the songs.


## TODO

  * Don't hard-code ".mp3" for song files
  * Limit memoization cache size
  * Give more options to the command line, such as bit-rate limit for low bandwidth
  * Test on more subsonic servers (only demo.subsonic.org and Airsonic have been tested yet)
  * Clean-up the code!
  * Fix bugs!
