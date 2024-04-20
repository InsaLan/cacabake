# cacabake

Rust script for baking (or pre-rendering) videos to text using libcaca, and then playing them. For UNIX-based systems only. No sound.

In my rough testing, this showed to be ~5 times more efficient in cpu usage than `mpv --vo=caca` (efficiency difference probably depends on specs and video file) (this was NOT a waste of time !! yay)

Videos are baked to the size of the terminal that the command is run in. The video will be stretched to fit, so videos should match the aspect ratio of the terminal to look correct.

Press q to leave playback.

## Requirements

`ffmpeg` \
`libcaca` (this script uses `img2txt` specifically)

## Build & run

Build once with `cargo build --release` then run `./target/release/cacabake video.mp4` to create the `video.baked` file, and `./target/release/cacabake video.baked` to display it. Pass the `-l` argument to play on loop.