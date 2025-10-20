#!/usr/bin/env bash
set -e

# Force update this repo
git fetch origin
git reset --hard origin/main
git clean -fd