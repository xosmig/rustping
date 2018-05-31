#!/bin/sh -eu

echo "ping localhost ..."
sudo ./target/debug/rustping -c 1 -W 1 localhost | tail -1 | grep -Pi '^Ok' > /dev/null 2>/dev/null

echo "ping example.com ..."
sudo ./target/debug/rustping -c 1 -W 1 example.com | tail -1 | grep -Pi '^Ok' > /dev/null 2>/dev/null
