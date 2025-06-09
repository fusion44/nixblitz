# Run this in a shell with all glibc locales installed:
# nix-shell -p glibcLocales

def build_locale_db [] {
  let locnames = locale --all-locales | split row "\n"
  mut lines = [
    "//! Module providing a possible locale values",
	 "#![allow(dead_code)]\n",
	 "pub const LOCALES: &[&str] = &["
  ]

  for $n in $locnames {
    $lines = $lines ++ [$"  \"($n)\","]
  }

  $lines = $lines ++ ["];"]
  $lines | str join "\n" | save -f /home/f44/dev/blitz/nixblitz/main/packages/nixblitz_system/src/locales.rs
}

build_locale_db
