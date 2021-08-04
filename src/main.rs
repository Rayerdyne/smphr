mod smphr;

fn main() {
    match smphr::SmphrParams::from_args() {
        Ok(p) => {
            if let Err(e) = smphr::exec(p) {
                println!("{}", e);
            }
        },
        Err(e) => {
            println!("Error occured: {}", e)
        }
    }
}
