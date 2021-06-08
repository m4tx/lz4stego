lz4stego
========

An implementation of a bit recycling steganography for the LZ4 compression algorithm.

## Description

LZ4 format internally represents chunks of data either as literals, or matches of arbitrary lengths from some previous locations in the file. This allows it to achieve very high compression and decompression speed while maintaining decent compression ratio.

However, since it does not use Huffman encoding to encode match location offsets (like for instance Deflate algorithm does), it allows one to choose some specific match each time in the file without sacrificing compression ratio. The technique is being called the bit recycling and is the base of what happens in lz4stego. Each time a 4-byte substring has appeared in the file at least twice, we can choose one of the previous matches of our choosing, thus hiding some bits of data. Most decompressors won't ever see any difference, but lz4stego decompressor can bring back the original hidden message.   

## Usage

### Compressing

```
lz4stego -i <hidden_file_path> <input_file_path> <output_file_path>
```

You can optionally use `-c/--count` flag to tell lz4stego to output the maximum possible number of bytes that can be hidden.

`-p/--prefer-hidden` flag tells lz4stego to sacrifice compression ratio and try to output as many bytes of hidden data as possible. This is achieved by ignoring the match lengths and not trying to output the longest match - any match that is at least 4 bytes long is usable. Note that in this mode, the compression ratio depends on the actual contents of the hidden data.

### Decompressing

```
lz4stego -d -i <hidden_file_path> <input_file_path> <output_file_path>
```

Please note that if you used `-p/--prefer-hidden` flag for compressing, it must also be used when decompressing data.

## Steganography benchmark

| Dataset                               | Original size | Compressed size | Hidden capacity | Hidden capacity with `-p` flag |
|---------------------------------------|---------------|-----------------|-----------------|--------------------------------|
| First 1MB of [enwik8][1]              | 1,000,000B    | 428,658B        | 17,320B         | 60,041B                        |
| 4MiB output of /dev/urandom           | 4,194,304B    | 4,194,327B      | 0B              | 0B                             |
| sao file from [Silesia Corpus][2]     | 7,251,944B    | 5,808,448B      | 141,736B        | 280,296B                       |
| reymont file from [Silesia Corpus][2] | 6,627,202B    | 2,245,208B      | 98,789B         | 672,126B                       |

[1]: https://cs.fit.edu/~mmahoney/compression/textdata.html
[2]: http://sun.aei.polsl.pl/~sdeor/index.php?page=silesia

## Building
lz4stego is being built using [Cargo and Rust stable](https://www.rust-lang.org/tools/install).

```
cargo build --release
```

The binary will be created in `target/release/lz4stego`.
