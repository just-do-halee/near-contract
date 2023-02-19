#!/bin/bash
set -e # if any command fails, exit the script


git clone https://github.com/just-do-halee/near-contract

cd near-contract

rm -rf .git

rm LICENSE
rm LICENSE-APACHE

if [[ "$OSTYPE" == "msys" || "$OSTYPE" == "cygwin" ]]; then
    # Windows with Cygwin or MSYS2
    NULPATH=NUL
else
    NULPATH=/dev/null
fi

# git init

git --version > $NULPATH
if [[ $? == 0 ]]; then
    git init
fi

# RUST

rustup --help > $NULPATH
if [[ $? != 0 ]]; then
    # if there is no rustup, install it
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
else
    # there is rustup, so update it
    rustup update
fi

# NPM

npm --version > $NULPATH
if [[ $? != 0 ]]; then
    # if there is no npm, install it
    curl -L https://www.npmjs.com/install.sh | sh
else
    # there is npm, so update it
    npm install -g npm
fi

# NEAR

near --help > $NULPATH
if [[ $? != 0 ]]; then
    # if there is no near, install it
    npm install -g near-cli
else
    # there is near, so update it
    npm update -g near-cli
fi

near login
