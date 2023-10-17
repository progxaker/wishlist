#!/bin/bash

echo "Running crontab"
cron

echo "Running Wishlist"
RUST_BACKTRACE=1 /usr/local/bin/wishlist serve
