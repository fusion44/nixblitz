alias nginxjournal = journalctl -u nginx.service
alias nginxjournalf = journalctl -n 50 -fu nginx.service
alias nginxbat = systemctl cat nginx.service

# Prints the current nginx.conf file in use using the bat command
def nginxbatconf [] {
  let res = open /etc/systemd/system/nginx.service
  $res | split row "\n" | where { str starts-with 'ExecStart=/nix/store' } | first | split row "'" | get 1 | bat $in
}

alias norupojournal = sudo journalctl -u nixblitz-norupo.service
alias norupojournalf = sudo journalctl -n 50 -fu nixblitz-norupo.service
alias norupobat = systemctl cat nixblitz-norupo.service

alias installenginejournal = sudo journalctl -u nixblitz-install-engine.service
alias installenginejournalf = sudo journalctl -n 50 -fu nixblitz-install-engine.service
alias installengineservicebat = systemctl cat nixblitz-install-engine.service

alias systemenginejournal = sudo journalctl -u nixblitz-system-engine.service
alias systemenginejournalf = sudo journalctl -n 50 -fu nixblitz-system-engine.service
alias systemengineservicebat = systemctl cat nixblitz-system-engine.service
