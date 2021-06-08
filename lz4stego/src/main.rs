use clap::{App, Arg};
use log::{Level, LevelFilter, Metadata, Record, SetLoggerError};

mod library_ctrl;

pub const PROJ_NAME: &str = env!("CARGO_PKG_NAME");
pub const PROJ_VERSION: &str = env!("CARGO_PKG_VERSION");
pub const PROJ_AUTHORS: &str = env!("CARGO_PKG_AUTHORS");

struct Logger;

impl log::Log for Logger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.level() <= Level::Debug
    }

    fn log(&self, record: &Record) {
        if self.enabled(record.metadata()) {
            println!(
                "[{}] [{}] {}",
                record.level(),
                record.target(),
                record.args()
            );
        }
    }

    fn flush(&self) {}
}

static LOGGER: Logger = Logger;

pub fn init_logging() -> Result<(), SetLoggerError> {
    log::set_logger(&LOGGER).map(|()| log::set_max_level(LevelFilter::Debug))
}

fn main() {
    let matches = App::new(PROJ_NAME)
        .version(PROJ_VERSION)
        .author(PROJ_AUTHORS)
        .about("LZ4 compressor and decompressor with steganography")
        .arg(
            Arg::with_name("decompress")
                .short("d")
                .long("decompress")
                .help("Decompress instead of compressing"),
        )
        .arg(
            Arg::with_name("count")
                .short("c")
                .long("count")
                .help("Count how many bytes of data can be hidden"),
        )
        .arg(
            Arg::with_name("hidden")
                .short("i")
                .long("hidden")
                .value_name("FILE")
                .help("Hidden data file path"),
        )
        .arg(
            Arg::with_name("prefer-hidden")
                .short("p")
                .long("prefer-hidden")
                .help("Prefer hidden data capacity over compression ratio. Must be set for decompressing as well"),
        )
        .arg(
            Arg::with_name("verbose")
                .short("v")
                .long("verbose")
                .help("Verbose output"),
        )
        .arg(
            Arg::with_name("INPUT")
                .help("input filename")
                .required(true)
                .index(1),
        )
        .arg(
            Arg::with_name("OUTPUT")
                .help("output filename")
                .required(true)
                .index(2),
        )
        .get_matches();

    let input = matches.value_of("INPUT").unwrap();
    let output = matches.value_of("OUTPUT").unwrap();
    let hidden = matches.value_of("hidden");
    let decompress = matches.is_present("decompress");
    let count = matches.is_present("count");
    let prefer_hidden = matches.is_present("prefer-hidden");
    let verbose = matches.is_present("verbose");

    if verbose {
        init_logging().unwrap();
    }

    if decompress {
        library_ctrl::decompress(input, output, hidden, prefer_hidden);
    } else {
        library_ctrl::compress(input, output, hidden, count, prefer_hidden);
    }
}
