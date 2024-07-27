#!/bin/bash

cd /opt/dipak/dipakniroula.com.np

git pull


cargo build --release

sudo systemctl restart dipakniroula.service
sudo systemctl restart nginx
