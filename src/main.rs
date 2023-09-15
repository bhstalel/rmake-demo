mod rmake;

use std::fs::metadata;
use std::str::FromStr;
use structopt::StructOpt;
use tracing::{debug, error, info, Level};
use tracing_subscriber;

#[macro_export]
macro_rules! RMakeError {
    ($($message:expr),*) => {
        error!($($message),*);
        std::process::exit(1);
    };
}

#[derive(StructOpt)]
struct RMakeArgs {
    #[structopt(help = "Target")]
    target: Option<String>,

    #[structopt(long = "--directory", short = "-C", default_value = "./")]
    directory: String,
}

fn main() {
    let log_l = match std::env::var("LOGL") {
        Ok(ll) => ll,
        Err(_) => String::from("INFO"),
    };

    /* Prepare tracing */
    tracing::subscriber::set_global_default(
        tracing_subscriber::fmt()
            .with_target(false)
            .compact()
            .with_max_level(Level::from_str(&log_l).unwrap())
            .finish(),
    )
    .unwrap();

    /* Parse arguments */
    let rmake_args = RMakeArgs::from_args();
    let dir = rmake_args.directory;

    /* Check if given directory is directory */
    if !metadata(&dir).unwrap().is_dir() {
        RMakeError!("Path is not a directory: {}", dir);
    }

    /* Change current working directory */
    info!("Setting build directory ..");
    std::env::set_current_dir(&dir).expect(format!("Cannot change directory to: {}", dir).as_str());

    debug!("Current dir: {:?}", std::env::current_dir().unwrap());

    let rmake = rmake::rmake::RMake::new("RMakefile.yml".to_string());
    match rmake {
        Ok(mut rm) => rm.run(rmake_args.target),
        Err(e) => {
            RMakeError!("Error loading RMakefile.yml file : {}", e);
        }
    }
}
