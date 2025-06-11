pub fn format_bytes_to_gb(bytes: u64) -> String {
    let gigabytes = bytes as f64 / 1_000_000_000.0;
    format!("{:.2} GB", gigabytes)
}
