fn main() -> Result<(), Box<dyn std::error::Error>> {
    tonic_build::configure()
        .out_dir("./src/pb")
        .compile(&["live_room.proto"], &["proto/streamserver"])?;
    Ok(())
}
