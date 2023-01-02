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

(
  cd ~/src/lunarvim_rust/
  podman build -t lunarvim_rust .
)

mkdir -p ~/.local/rust_docker_cargo-smsbar-images-dev
chcon -R -t container_file_t ~/.local/rust_docker_cargo-smsbar-images-dev

podman kill smsbar-images-dev || true
podman rm smsbar-images-dev || true

# See https://github.com/xd009642/tarpaulin/issues/1087 for the seccomp thing
podman run --detach --name smsbar-images-dev --security-opt seccomp=~/src/lunarvim_rust/seccomp.json -w /root/src/smsbar-images \
  -v ~/src:/root/src -v ~/.local/rust_docker_cargo-smsbar-images-dev:/root/.cargo \
  -v ~/config/dotfiles/lunarvim:/root/.config/lvim  -v ~/config/dotfiles/bashrc:/root/.bashrc \
  -v ~/config/dotfiles/bothrc:/root/.bothrc \
  -it lunarvim_rust bash /root/src/smsbar-images/dev_run_inside.sh

echo "Copying latest SMS backup into /tmp/"
podman cp "$(ls -rt ~/Dropbox/Apps/SMSBackupRestore/*sms* | tail -n 1)" smsbar-images-dev:/tmp/sms.xml

echo "execing a shell for you"
podman exec -it smsbar-images-dev bash || true

echo "stopping the container"
podman kill smsbar-images-dev
