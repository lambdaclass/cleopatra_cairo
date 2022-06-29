#!/bin/bash

test_files=($(ls -p ../cairo_programs | grep -v / | sed -E 's/\.cairo//'))

rm cairo_programs/*.json

for file in ${test_files[@]}; do
    cairo-compile "../cairo_programs/$file.cairo" --output "../cairo_programs/$file.json"
done