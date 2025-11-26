#!/bin/bash

set -e

mkdir -p assets
cd assets
wget https://raw.githubusercontent.com/alecjacobson/common-3d-test-models/refs/heads/master/data/suzanne.obj
wget https://raw.githubusercontent.com/alecjacobson/common-3d-test-models/refs/heads/master/data/stanford-bunny.obj 
wget https://raw.githubusercontent.com/alecjacobson/common-3d-test-models/refs/heads/master/data/xyzrgb_dragon.obj 
mv xyzrgb_dragon.obj xyzrgb-dragon.obj
cd ../
