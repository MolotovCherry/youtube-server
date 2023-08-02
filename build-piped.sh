#!/usr/bin/env bash

if [ ! -d "piped/node_modules" ]; then
    pnpm -C piped install
fi

if [ ! -d "piped/dist" ]; then
    pnpm -C piped build
fi
