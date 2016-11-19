extern crate argparse;
#[macro_use]
extern crate enum_primitive;
extern crate num;
#[macro_use]
extern crate log;
extern crate env_logger;
extern crate byteorder;

mod utils;
mod header;
mod runtime;
mod interpreter;
mod instructions;
mod constants;
mod raptor_object;

use std::env;
use env_logger::LogBuilder;
use log::{LogRecord, LogLevelFilter};
use argparse::{ArgumentParser, StoreTrue, Store, Print};

use runtime::Runtime;

const DEFAULT_LOG_LEVEL: LogLevelFilter = LogLevelFilter::Debug;
pub static ACCEPTABLE_EXTENSIONS: [&'static str; 2] = ["crap", "crapt"];

#[derive(Default, Debug)]
pub struct Options {
    debug: bool,
    input: String,
}

fn main() {
    let mut options = Options {
        debug: true,
        input: String::new(),
    };
    {   // this block limits the scope of borrows from ap.refer() calls
        let mut ap = ArgumentParser::new();
        ap.set_description("RaptorScript Runtime/Interpreter.");
        ap.refer(&mut options.debug)
            .add_option(&["-d", "--debug"], StoreTrue,
            "print every interpreted instruction");
        ap.refer(&mut options.input)
            .add_option(&["-i", "--input"], Store,
            "input bytecode file");
        ap.add_option(&["-v", "--version"],
            Print(env!("CARGO_PKG_VERSION").to_string()),
            "show version");
        ap.parse_args_or_exit();
    }

    // Logging stuff
    let format = |record: &LogRecord| {
        format!("[{}]: {}", record.level(), record.args())
    };

    let mut builder = LogBuilder::new();
    if env::var("RUST_LOG").is_ok() {
        builder.parse(&env::var("RUST_LOG").unwrap());
    } else {
        builder.format(format).filter(None, DEFAULT_LOG_LEVEL);
    }
    builder.init().unwrap();

    // Parse input, start runtime
    if !options.input.is_empty() {
        if utils::should_open(&options.input) {
            let data = utils::try_open_file(&options.input, options.debug);
            let mut runtime = Runtime::new(data, options);
            runtime.run();

        } else {
            warn!("Invalid input file extension. Accepted formats are .crapt and .crap");
            return;
        }
    } else {
        warn!("No input file given. Use -h or --help for help.");
        return;
    }

}
