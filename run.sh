#!/bin/bash

# Error trapping from https://gist.github.com/oldratlee/902ad9a398affca37bfcfab64612e7d1
__error_trapper() {
  local parent_lineno="$1"
  local code="$2"
  local commands="$3"
  echo "error exit status $code, at file $0 on or near line $parent_lineno: $commands"
}
trap '__error_trapper "${LINENO}/${BASH_LINENO}" "$?" "$BASH_COMMAND"' ERR

set -euE -o pipefail

# Cron's path tends to suck
export PATH=/usr/local/bin:/usr/local/sbin:/usr/bin:/usr/sbin:$HOME/bin:$HOME/.local/bin

selfdir="$(readlink -f "$(dirname "$0")")"
cd "$selfdir"

(
  cd ~/src/lunarvim_rust/
  podman build -t lunarvim_rust .
)

mkdir -p ~/.local/rust_docker_cargo-smsbar-images
chcon -R -t container_file_t ~/.local/rust_docker_cargo-smsbar-images

podman kill smsbar-images || true
podman rm smsbar-images || true

# See https://github.com/xd009642/tarpaulin/issues/1087 for the seccomp thing
podman run --detach --name smsbar-images --security-opt seccomp=~/src/lunarvim_rust/seccomp.json -w /root/src/smsbar-images \
  -v ~/src:/root/src -v ~/.local/rust_docker_cargo-smsbar-images:/root/.cargo \
  -v ~/config/dotfiles/lunarvim:/root/.config/lvim  -v ~/config/dotfiles/bashrc:/root/.bashrc \
  -v ~/config/dotfiles/bothrc:/root/.bothrc \
  -it lunarvim_rust bash /root/src/smsbar-images/dev_run_inside.sh

echo "Copying latest SMS backup into /tmp/"
podman cp "$(ls -rt ~/Dropbox/Apps/SMSBackupRestore/*sms* | tail -n 1)" smsbar-images:/tmp/sms.xml

echo "cleaning old images"
rm -rf output/
mkdir output

echo "running image extractor"
podman exec -it smsbar-images cargo run /tmp/sms.xml "$@"

echo "stopping the container"
podman kill smsbar-images
podman rm smsbar-images

echo "copying images"
rsync -av output/ ~/Dropbox/Pictures/ZF_Prep/SMS_Images/
