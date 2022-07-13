#!/usr/bin/env python3

import sys

def main():
    filename1 = sys.argv[1]
    filename2 = sys.argv[2]
    cairo_mem = {}
    cleo_mem = {}

    with open(filename1, 'rb') as f:
        cairo_raw = f.read()
        assert len(cairo_raw) % 40 == 0, f'{filename1}: malformed memory file from Cairo VM'
        chunks = len(cairo_raw) // 40
        for i in range(0, chunks):
            chunk = cairo_raw[i*40:(i+1)*40]
            k, v = int.from_bytes(chunk[:8], 'little'), int.from_bytes(chunk[8:], 'little')
            assert k not in cairo_mem, f'{filename1}: address {k} has two values'
            cairo_mem[k] = v
        assert len(cairo_mem) * 40 == len(cairo_raw), f'{filename1}: {len(cairo_mem) * 40} != {len(cairo_raw)}'

    with open(filename2, 'rb') as f:
        cleo_raw = f.read()
        assert len(cleo_raw) % 40 == 0, f'{filename2}: malformed memory file from Cleopatra VM'
        chunks = len(cleo_raw) // 40
        for i in range(0, chunks):
            chunk = cleo_raw[i*40:(i+1)*40]
            k, v = int.from_bytes(chunk[:8], 'little'), int.from_bytes(chunk[8:], 'little')
            assert k not in cleo_mem, f'{filename2}: address {k} has two values'
            cleo_mem[k] = v
        assert len(cleo_mem) * 40 == len(cleo_raw), f'{filename2}: {len(cleo_mem) * 40} != {len(cleo_raw)}'

    assert len(cairo_mem) == len(cleo_mem), f'{filename2}: {len(cairo_mem)=} {len(cleo_mem)=}'
    if cairo_mem != cleo_mem:
        print(f'Mismatch between {filename1} (Cairo) and {filename2} (Cleopatra)')
        print('keys in Cairo but not Cleopatra:')
        for k in cairo_mem:
            if k in cleo_mem:
                continue
            print(f'{k}:{v}')
        print('keys in Cleopatra but not Cairo:')
        for k in cleo_mem:
            if k in cairo_mem:
                continue
            print(f'{k}:{v}')
        print('mismatched values (Cairo <-> Cleopatra)):')
        for k in cleo_mem:
            if k not in cairo_mem:
                continue
            if cleo_mem[k] == cairo_mem[k]:
                continue
            print(f'{k}:({cairo_mem[k]} <-> {cleo_mem[k]})')
        exit(1)


if __name__ == '__main__':
    main()

