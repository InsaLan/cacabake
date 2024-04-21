# cacabake

Rust script for baking (or pre-rendering) videos to text using libcaca, and then playing them. No sound.

In my rough testing, this showed to be ~5 times more efficient in cpu usage than `mpv --vo=caca` (this was NOT a waste of time !! yay)

However, baked text files often weigh 3-5 times more than the original video file, depending on compression.

## Requirements

`ffmpeg` \
`libcaca` (this script uses `img2txt` specifically)

## Build & run

Build once with `cargo build --release` then run `./target/release/cacabake video.mp4` to create the `video.baked` file, and `./target/release/cacabake video.baked` to display it. Pass the `-l` argument to play on loop. Press q to leave playback.

Videos are baked to the size of the terminal that the command is run in. The video will be stretched to fit, so videos should match the aspect ratio of the terminal to look correct.
