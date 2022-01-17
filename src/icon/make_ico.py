#!/usr/bin/python3

import sys
import os
import shutil
import subprocess


sizes = [16, 24, 32, 48, 64, 72, 96, 128, 180, 256]

if len(sys.argv) < 2 or not os.path.isfile(sys.argv[1]):
    print('Not enough arguments or given argument is not a file')
    sys.exit(1)

if os.path.isdir('ico_sizes_temp'):
    shutil.rmtree('ico_sizes_temp')
os.mkdir('ico_sizes_temp')
for size in sizes:
    subprocess.run(['inkscape', '-w', str(size), '-o', os.path.join('ico_sizes_temp', f'{size}.png'), sys.argv[1]])
subprocess.run(['magick', 'convert'] + [os.path.join('ico_sizes_temp', f'{size}.png') for size in sizes] + [os.path.splitext(sys.argv[1])[0] + '.ico'])