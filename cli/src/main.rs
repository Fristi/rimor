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

    /// Allow invalid UTF-8 paths
    #[arg(short = 'I', value_name = "FILE", value_hint = clap::ValueHint::DirPath, required = true)]
    file: std::path::PathBuf
}



fn main() {

    let args = Args::parse();

    let x = args.x.unwrap_or(0);
    let y = args.y.unwrap_or(0);
    let graph = Graph::from_file(&args.file);
    let recovery_rate = args.recovery_rate.unwrap_or(1);
    let path = graph.path_planning_bfs((x, y), args.max_timesteps, recovery_rate);

    println!("Path: {path:?}");
    println!("Score: {:?}", path.score())
}
