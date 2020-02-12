use rtt1::builder;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("{:?}", builder::build());
    Ok(())
}
