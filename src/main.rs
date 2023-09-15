mod rmake;

use std::str::FromStr;
use tracing::{error, info, warn, Level};
use tracing_subscriber;

#[macro_export]
macro_rules! RMakeError {
    ($($message:expr),*) => {
        error!($($message),*);
        std::process::exit(1);
    };
}

fn main() {
    let log_l = match std::env::var("LOGL") {
        Ok(ll) => ll,
        Err(_) => String::from("INFO"),
    };

    /* Prepare tracing */
    // Prepare tracing
    tracing::subscriber::set_global_default(
        tracing_subscriber::fmt()
            .with_target(false)
            .compact()
            .with_max_level(Level::from_str(&log_l).unwrap())
            .finish(),
    )
    .unwrap();

    let rmake = rmake::rmake::RMake::new(String::from("examples/RMakefile.yml"));
    if let Ok(mut rm) = rmake {
        rm.run("main".to_string());
    }
}
