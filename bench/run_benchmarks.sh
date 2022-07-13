#!/usr/bin/env sh
tests_path="../cairo_programs/benchmarks"

set -e
echo "Cleaning old results"
rm -f results

for file in $(ls $tests_path | grep .cairo | sed -E 's/\.cairo//'); do
    echo "Running $file benchmark"
    echo "\n*** $file.cairo times ***" >> results

    pyenv global 3.7.12

    cairo_time=$( (time cairo-run --layout all --program $tests_path/$file.json) 2>&1 &)
    echo "\nOriginal Cairo VM $file.cairo:" >> results
    echo "$cairo_time" >> results

    pyenv global pypy3.7-7.3.9

    cairo_pypy_time=$( (time cairo-run --layout all --program $tests_path/$file.json) 2>&1 &)
    echo "\nPyPy Cairo VM $file.cairo:" >> results
    echo "$cairo_pypy_time" >> results

    pyenv global 3.7.12

    cleo_time=$( (time ../target/release/cleopatra-run $tests_path/$file.json) 2>&1 &)
    echo "\nRust Cleopatra VM $file.cairo:" >> results
    echo "$cleo_time" >> results
done

cat results

echo "Cleaning results"
rm -f results
