use rtt1::args::subcommands;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("{:?}", subcommands());
    Ok(())
}
