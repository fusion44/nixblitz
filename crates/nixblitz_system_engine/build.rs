use vergen::{BuildBuilder, Emitter};
use vergen_git2::Git2Builder;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut emitter = Emitter::default();
    emitter.add_instructions(&BuildBuilder::all_build()?)?;
    if std::env::var("VERGEN_IDEMPOTENT").is_err() {
        println!(
            "cargo:warning=VERGEN_IDEMPOTENT not set, attempting to generate Git info via vergen."
        );
        let git_instructions = Git2Builder::default()
            .sha(true)
            .describe(true, true, None)
            .dirty(true)
            .build()?;
        let _ = emitter.add_instructions(&git_instructions);
    } else {
        println!(
            "cargo:warning=VERGEN_IDEMPOTENT is set, skipping vergen's Git info generation. Expecting Nix to provide VERGEN_GIT_SHA, etc."
        );
    }

    let _ = emitter.emit();

    Ok(())
}
