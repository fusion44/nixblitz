# Main function to update the default.nix file and rebuild.
#
# Usage:
#   nu ./update-default-nix.nu
#   nu ./update-default-nix.nu <your-commit-sha>
def main [
  commit_sha?: string # The optional commit SHA to use. If not provided, it uses the latest git commit.
] {
    # --- 1. Determine the Commit SHA ---
    # Use the provided commit_sha or get the latest one from the local git repository.
    let new_commit_sha = if ($commit_sha | is-empty) {
        print "INFO: No commit SHA provided, using latest from git."
        git rev-parse HEAD | str trim
    } else {
        $commit_sha
    }

    print $"INFO: Using commit SHA: ($new_commit_sha)"

    let file_path = "default.nix"

    # --- 2. Update commitSha in default.nix ---
    print $"INFO: Updating 'commitSha' in ($file_path)..."

    # Read the file content.
    let content = open $file_path

    # Define the regex to find the commitSha line.
    let commit_sha_regex = 'commitSha = ".*?"'
    let new_commit_sha_line = $'commitSha = "($new_commit_sha)"'

    # Replace the old commit SHA with the new one.
    let updated_content = $content | str replace --regex $commit_sha_regex $new_commit_sha_line
    $updated_content | save -f $file_path

    print "SUCCESS: 'commitSha' updated."

    # --- 3. Run nix build to get the new sha256 ---
    print "INFO: Running 'nix build' to get the new sha256 hash. This is expected to fail."

    # Temporarily set the sha256 to an empty string to guarantee a hash mismatch error.
    let content_for_build = $updated_content | str replace --regex 'sha256 = ".*?"' 'sha256 = ""'
    $content_for_build | save -f $file_path

    # Run the build command and capture the output (stdout and stderr).
    let build_result =  nix build | complete
    let stderr = $build_result.stderr

    if $build_result.exit_code == 0 {
        print "WARNING: Nix build succeeded unexpectedly. The sha256 might already be correct."
        return
    }

    # --- 4. Extract the new sha256 from the error output ---
    print "INFO: Extracting new sha256 from the error message..."

    # This captures only the 'sha256-...' part.
    let new_sha256 = ($stderr | parse -r `got:\s*(sha256-\S+)` | get capture0 | first | default "")

    if ($new_sha256 | is-empty) {
        print "ERROR: Could not find the new sha256 in the nix build output."
        print "--- NIX STDERR ---"
        print $stderr
        print "------------------"
        # Restore the original file content before exiting
        $content | save -f $file_path
        exit 1
    }

    print $"SUCCESS: Found new sha256: ($new_sha256)"

    # --- 5. Update sha256 in default.nix ---
    let sha256_regex = 'sha256 = ".*?"'
    let new_sha256_line = $'sha256 = "($new_sha256)"'

    # Use the content that already has the updated commit SHA
    let final_content = $updated_content | str replace --regex $sha256_regex $new_sha256_line
    $final_content | save -f $file_path

    print "SUCCESS: 'sha256' updated."

    # --- 6. Run nix build again ---
    print "INFO: Running 'nix build' again to verify..."

    let final_build = nix build | complete

    if $final_build.exit_code == 0 {
        print "✅ SUCCESS: Nix build completed successfully!"
    } else {
        print "ERROR: Final nix build failed."
        print $final_build.stderr
        exit 1
    }
}
