#!/bin/bash

# cargo install --version=0.6.0 sqlx-cli --no-default-features --features postgres,rustls

apt-get update
apt-get install -y locales locales-all

sed -i '/PackerComplete/d' ~/.bothrc-local

. ~/.bashrc

echo "doing lua/lvim reload"
lvim -c "lua require('lvim.core.log'):set_level([[info]])" -c 'autocmd User PackerComplete quitall' -c 'LvimCacheReset' -c 'PackerSync'
echo
echo

sleep 999999
