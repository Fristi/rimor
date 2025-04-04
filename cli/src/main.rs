use std::sync::mpsc;
use std::thread;
use std::time::Duration;
use clap::*;
use pathfinding::*;

#[derive(Parser, Debug)] // requires `derive` feature
#[command(term_width = 0)] // Just to make testing across clap features easier
struct Args {
    #[arg(short = 'x')]
    x: Option<usize>,

    #[arg(short = 'y')]
    y: Option<usize>,

    #[arg(short = 'T', required = true)]
    max_timesteps: u32,

    #[arg(short = 'R')]
    recovery_rate: Option<u32>,

    #[arg(short, long, value_parser = parse_duration)]
    timeout: Option<Duration>,

    /// Allow invalid UTF-8 paths
    #[arg(short = 'I', value_name = "FILE", value_hint = clap::ValueHint::DirPath, required = true)]
    file: std::path::PathBuf
}

fn parse_duration(s: &str) -> Result<Duration, &'static str> {
    s.parse::<u64>()
        .map(Duration::from_millis)
        .map_err(|_| "Invalid duration, expected a positive integer")
}



fn main() {

    let args = Args::parse();

    let x = args.x.unwrap_or(0);
    let y = args.y.unwrap_or(0);
    let graph = Graph::from_file(&args.file);
    let recovery_rate = args.recovery_rate.unwrap_or(1);
    let timeout = args.timeout.unwrap_or(Duration::from_secs(2));

    let (tx, rx) = mpsc::channel();

    // Spawn the function in a separate thread
    thread::spawn(move || {
        let result = graph.path_planning_bfs((x, y), args.max_timesteps, recovery_rate);
        let _ = tx.send(result); // Send result through the channel
    });

    // Set a timeout duration
    match rx.recv_timeout(timeout) {
        Ok(path) => {
            println!("Path: {path:?}");
            println!("Score: {:?}", path.score())
        }
        Err(mpsc::RecvTimeoutError::Timeout) => {
            println!("Timed out after {:?}", timeout);
        }
        Err(mpsc::RecvTimeoutError::Disconnected) => {
            println!("Thread disconnected");
        }
    }
}
