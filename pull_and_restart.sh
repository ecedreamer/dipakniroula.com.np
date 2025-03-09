#!/bin/bash

cd /opt/dipak_site/dipakniroula.com.np

git pull


cargo build --release
export RUST_LOG=info

sudo systemctl restart dipakniroula.service
sudo systemctl restart nginx
