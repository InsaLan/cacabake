// Made by pixup1 for Insalan XIX

// std
use std::env;
use std::fs::*;
use std::io::{Write, stdout};
use std::panic;
use std::path::Path;
use std::process::Command;
use std::thread;
use std::time::Duration;

// crates
use getopts::Options;

use indicatif::ProgressBar;

use crossterm;

use evalexpr::*;

use tokio::task;

fn check_dependencies() {
	if Command::new("ffmpeg").arg("-version").output().is_err()
		|| Command::new("ffprobe").arg("-version").output().is_err() {
		panic!("FFmpeg is not installed or not in PATH. Please visit https://ffmpeg.org/download.html to see how to install it for your operating system.");
	}
	if Command::new("img2txt").arg("--help").output().is_err() {
		panic!("libcaca is not installed or not in PATH. Please find a way to install it for your operating system.");
	}
}

async fn bake_video(spath: &Path, quiet: bool) {
	check_dependencies();

	let tmppath = Path::new("/tmp/cacabake");
	
	if tmppath.exists() {
		remove_dir_all(tmppath).expect("Failed to remove /tmp/cacabake directory");
	}
	create_dir_all(tmppath).expect("Failed to create /tmp/cacabake directory");
	
	if !quiet { println!("Getting framerate..."); }

	let ffprobe_output = Command::new("ffprobe")
		.arg("-v")
		.arg("error")
		.arg("-select_streams")
		.arg("v:0")
		.arg("-show_entries")
		.arg("stream=avg_frame_rate")
		.arg("-of")
		.arg("default=noprint_wrappers=1:nokey=1")
		.arg(spath)
		.output()
		.expect("FFprobe error");

	if !ffprobe_output.status.success() {
		panic!(
			"FFprobe error: {}",
			String::from_utf8_lossy(&ffprobe_output.stderr)
		);
	}

	let framerate_expr = String::from_utf8(ffprobe_output.stdout)
		.expect("FFprobe output was not valid UTF-8")
		.trim()
		.to_string();

	if framerate_expr.is_empty() {
		panic!("No video stream found in input file");
	}

	let framerate = eval(&framerate_expr).expect("Error evaluating framerate"); // Framerate is given as a fraction, we need a number so we use the `evalexpr` crate
	
	if spath.with_extension("baked").exists() {
		remove_file(spath.with_extension("baked")).expect("Failed to remove existing baked file");
	}
	
	let mut outfile = OpenOptions::new()
		.create(true)
		.append(true)
		.open(spath.with_extension("baked")).expect("Failed to create output file");
	
	write!(outfile, "{}", framerate).expect("Failed to write to file");
	
	create_dir_all(tmppath.join("frames")).expect("Failed to create /tmp/cacabake/frames directory");
	
	if !quiet { println!("Extracting frames..."); }

	let ffmpeg_output = Command::new("ffmpeg")
		.arg("-i")
		.arg(spath)
		.arg("-loglevel")
		.arg("quiet")
		.arg(tmppath.join("frames/%015d.png").to_str().unwrap())
		.output()
		.expect("FFmpeg error");

	if !ffmpeg_output.status.success() {
		panic!(
			"FFmpeg error: {}",
			String::from_utf8_lossy(&ffmpeg_output.stderr)
		);
	}
	
	let tsize = crossterm::terminal::size().expect("Couldn't get terminal size");
	
	let mut framepaths: Vec<_> = read_dir(tmppath.join("frames")).unwrap().map(|r| r.unwrap()).collect();
	
	framepaths.sort_by_key(|dir| dir.path());

	let pb = ProgressBar::new(read_dir(tmppath.join("frames")).unwrap().by_ref().count() as u64);
	if !quiet { println!("Rendering frames ..."); }
	for framepath in framepaths {
		write!(outfile, "ඞ").unwrap();
		Command::new("img2txt")
			.args(["-W", &tsize.0.to_string()])
			.args(["-H", &tsize.1.to_string()])
			.args(["-d", "none"])
			.arg(framepath.path().to_str().unwrap())
			.stdout(outfile.try_clone().expect("Couldn't clone outfile"))
			.output() // We don't care about the output but we need to wait for the command to finish
			.expect("Error running img2txt. Check if libcaca is installed");
		pb.inc(1);
	}
	pb.finish_with_message("done");
	
	//remove_dir_all(tmppath).expect("Failed to remove /tmp/cacabake directory");
	if !quiet { println!("Baked to {} successfully !", spath.with_extension("baked").to_str().unwrap()); }
}

async fn play_video(spath: &Path, quiet: bool, lop: bool, any_key: bool) {
	if !quiet { println!("Loading..."); }
	let mut out = stdout();
	
	crossterm::terminal::enable_raw_mode().unwrap();
	crossterm::execute!(out, crossterm::cursor::Hide).unwrap();
	crossterm::execute!(out, crossterm::terminal::DisableLineWrap).unwrap();
	crossterm::execute!(out, crossterm::terminal::EnterAlternateScreen).unwrap();
	
	// This will be called on a panic so the terminal doesn't stay all messed up
	panic::set_hook(Box::new(|info| {
		let mut stdout = std::io::stdout();

		crossterm::terminal::disable_raw_mode().unwrap();
		crossterm::execute!(stdout, crossterm::terminal::EnableLineWrap).unwrap();
		crossterm::execute!(stdout, crossterm::terminal::LeaveAlternateScreen).unwrap();
		crossterm::execute!(stdout, crossterm::cursor::Show).unwrap();

		println!("{}", info);
	}));
	
	'outer: loop {
		let file_content = read_to_string(spath).expect("Failed to read baked file");
		let mut frames = file_content.split("ඞ").map(|s| s.to_string()); // Map the &str to Strings to avoid lifetime issues (???)
		let framerate: f64 = frames.next().unwrap().parse().unwrap();

		for frame in frames {
			//crossterm::execute!(stdout, crossterm::terminal::Clear(crossterm::terminal::ClearType::All)).unwrap(); // This almost works...
			crossterm::execute!(out, crossterm::cursor::MoveTo(0, 0)).unwrap();
			
			let print_task = task::spawn(async move {
				print!("{}", &frame.trim_end_matches('\n'));
			});
			let tempo_task = task::spawn(async move {
				thread::sleep(Duration::from_secs_f64(1.0 / framerate));
			});
			
			tokio::try_join!(print_task, tempo_task).unwrap(); // Wait for both tasks to finish
			
			if crossterm::event::poll(std::time::Duration::from_secs(0)).unwrap() {
				if let crossterm::event::Event::Key(key_event) = crossterm::event::read().unwrap() {
					if key_event.code == crossterm::event::KeyCode::Char('q') || any_key {
						crossterm::execute!(out, crossterm::terminal::LeaveAlternateScreen).unwrap();
						if !quiet { println!("Playback interrupted by user"); }
						break 'outer;
					}
				}
			}
		}

		if !lop {
			crossterm::execute!(out, crossterm::terminal::LeaveAlternateScreen).unwrap();
			if !quiet { println!("Reached end of video"); }
			break;
		}
	}
	crossterm::terminal::disable_raw_mode().unwrap();
	crossterm::execute!(out, crossterm::terminal::LeaveAlternateScreen).unwrap();
	crossterm::execute!(out, crossterm::cursor::Show).unwrap();    
	crossterm::execute!(out, crossterm::terminal::EnableLineWrap).unwrap();
}

fn print_usage(program: &str, opts: Options) {
	let brief = format!("Usage: {} FILE [options]", program);
	println!("{}", opts.usage(&brief));
}

#[tokio::main]
async fn main() {
	// getopts things
	let args: Vec<String> = env::args().collect();
	let program = args[0].clone();
	let mut opts = Options::new();
	
	opts.optflag("l", "loop", "play on loop (if input is baked file)");
	opts.optflag("a", "any", "any key press will exit playback");
	opts.optflag("q", "quiet", "quiet mode, no output other than video");
	opts.optflag("h", "help", "print this help menu");
	
	let matches = match opts.parse(&args[1..]) {
		Ok(m) => { m }
		Err(f) => { panic!("{}",f.to_string()) }
	};
	
	if matches.opt_present("h") {
		print_usage(&program, opts);
		return;
	}
	
	let mut lop = false;
	if matches.opt_present("l") {
		lop = true;
	}
	
	let mut any_key = false;
	if matches.opt_present("a") {
		any_key = true;
	}
	
	let mut quiet = false;
	if matches.opt_present("a") {
		quiet = true;
	}
	
	let spath = if !matches.free.is_empty() {
		Path::new(matches.free[0].as_str())
	} else {
		print_usage(&program, opts);
		return;
	};
	
	if !(spath.exists()) {
		println!("File not found");
		return;
	}
	
	let video_types = vec!["mp4", "mkv", "avi", "mov", "mpv"];
	
	if video_types.contains(&spath.extension().unwrap().to_string_lossy().as_ref()) {
		println!("Video will be baked to current terminal dimensions");
		bake_video(&spath, quiet).await;
	} else if spath.extension().unwrap().to_string_lossy() == "baked" { 
		play_video(&spath, quiet, lop, any_key).await;
	} else {
		println!("Invalid format, must be video or baked file");
		return;
	}
}