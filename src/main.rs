mod rmake;

fn main() {
    let rmake = rmake::rmake::RMake::new(String::from("examples/RMakefile.yml"));
    if let Ok(mut rm) = rmake {
        rm.run("main".to_string());
    }
}
