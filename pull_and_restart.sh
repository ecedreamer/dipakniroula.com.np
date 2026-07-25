#!/bin/bash
set -e

cd /opt/dipak_site/dipakniroula.com.np

git pull

export RUST_LOG=info

docker compose -f docker-compose.prod.yml build
docker compose -f docker-compose.prod.yml up -d
