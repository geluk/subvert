# subvert
Transform and clean up SRT subtitles

## Usage
```
Subvert 0.6
Johan Geluk <johan@geluk.io>
Transform and clean SRT subtitles

USAGE:
    subvert [OPTIONS]

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
    -b, --backup <FILE>         Write a backup of the original input to the specified file.
    -i, --input <FILE>          The file to read from. If not supplied, the subtitles will be read from standard input.
                                [default: -]
    -l, --leader-text <TEXT>    Insert the given text into the leader subtitle.
    -o, --output <FILE>         The file to write to. If not supplied, the subtitles will be written to standard output.
                                [default: -]

```
