use cucumber::{World as _, writer::JUnit};
use std::fs::File;

#[derive(Default, Debug, cucumber::World)]
struct World;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    std::fs::create_dir_all("target/junit")?;
    let out = File::create("target/junit/acceptance.xml")?;
    World::cucumber()
        .with_writer(JUnit::new(out, 0))
        .run("specs/features")
        .await;
    Ok(())
}
