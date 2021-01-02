#!/bin/bash

#This instruction provided by https://github.com/wzhd/vosk-sys
# Update dependencies
git submodule init
git submodule update
#First install dependencies and build the speech recognition toolkit Kaldi, which Vosk is based on
sudo apt-get install g++ automake autoconf unzip wget curl sox gfortran libtool subversion python2.7 python3 zlib1g-dev
#Build Kaldi
cd kaldi/tools
#Install linear algebra libraries, default - ATLAS
#Option 1: ATLAS installation
sudo apt-get install libatlas-base-dev
#Option 2: OpenBLAS installation
./extras/install_openblas.sh
#Option 3: Intel-mkl installation
./extras/install_mkl.sh
make
#Kaldi compilation
cd ../src
#Configure with mathlib set to the linear algebra library used:
./configure --mathlib=ATLAS --shared --use-cuda=no
make -j clean depend; make -j 4
#export path to kaldi root
cd ..
echo "export KALDI_ROOT=$(pwd)" >> $HOME/.bashrc
source ~/.bashrc
cd ../vosk-sys
cargo build --release