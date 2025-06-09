alias nginxjournal = journalctl -u nginx.service
alias nginxjournalf = journalctl -u nginx.service -f -n 50

# Prints the current nginx.conf file in use using the bat command
def nginxbatconf [] {
  let res = open /etc/systemd/system/nginx.service
  $res | split row "\n" | where { str starts-with 'ExecStart=/nix/store' } | first | split row "'" | get 1 | bat $in
}

alias webappjournal = sudo journalctl -u nixblitz-norupo.service
alias webappjournalf = sudo journalctl -u nixblitz-norupo.service -f -n 50
