use sp1_build::{build_program_with_args, BuildArgs};

fn main() {
    let args = BuildArgs {
        rustflags: vec!["--cfg".to_string(), "getrandom_backend=\"custom\"".to_string()],
        ..Default::default()
    };
    build_program_with_args("../zkvm", args);
}
