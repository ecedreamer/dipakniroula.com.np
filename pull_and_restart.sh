#!/bin/bash

cd /opt/dipak/dipakniroula.com.np

git pull


cargo build --release
export RUST_LOG=info

sudo systemctl restart dipakniroula.service
sudo systemctl restart nginx
