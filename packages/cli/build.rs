use vergen::{BuildBuilder, Emitter};
use vergen_git2::Git2Builder;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let _ = Emitter::default()
        .add_instructions(&BuildBuilder::all_build()?)?
        .add_instructions(&Git2Builder::all_git()?)?
        .emit();

    Ok(())
}
