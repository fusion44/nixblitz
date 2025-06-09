def build_tz_db [] {
  let tznames = timedatectl list-timezones --no-pager | split row "\n"
  mut lines = [
    "//! Module providing a possible timezone values",
	 "#![allow(dead_code)]\n",
	 "pub const TIMEZONES: &[&str] = &["
  ]

  for $n in $tznames {
    $lines = $lines ++ [$"  \"($n)\","]
  }

  $lines = $lines ++ ["];"]
  $lines | str join "\n" | save -f ../src/timezones.rs
}

build_tz_db
