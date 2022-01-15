# qoi-rs

Rust implementation of the [QOI (Quite OK) Image Format](https://qoiformat.org/) following the [File Format Specifications](https://qoiformat.org/qoi-specification.pdf).

**NOTE: This implementation can currently only encode a raw image to qoi. You can refer to the [C reference implementation](https://github.com/phoboslab/qoi) and use `qoiconv.c` to decode images or encode `png` images.**

## Usage

Build with `cargo build`. Run with `cargo run` or with the executable generated from the build.

```sh
./qoi-rs <input file> <width> <height> <channels (3/4)>
```

This will generate a `.qoi` output file.

## Generate test images

To turn a png into a raw image, use `qoiconv.c` from the [reference implementation](https://github.com/phoboslab/qoi), but add these lines to the last conditions that check the file format:

```c
} else if (STR_ENDS_WITH(argv[2], ".raw")) {
    FILE* fp = fopen(argv[2], "wb");
    fwrite(pixels, w * h * channels, 1, fp);
    fclose(fp);
    encoded = 1;
}
```
