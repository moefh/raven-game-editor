#!/bin/bash

git checkout pages &&
    git rebase main && \
    trunk build --release && \
    rm -rf ./docs/ && \
    mkdir -p ./docs/ && \
    cp dist/* docs/ && \
    git add docs/ && \
    git commit -m "Update pages"
