extern crate argparse;
extern crate byteorder;
#[macro_use] extern crate enum_primitive;
extern crate num;

#[macro_use] mod logger;
mod utils;
mod header;
mod interpreter;
mod instructions;

use interpreter::Interpreter;

use argparse::{ArgumentParser, StoreTrue, Store, Print};

pub static ACCEPTABLE_EXTENSIONS: [&'static str; 2] = ["crap", "crapt"];

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

    if !options.input.is_empty() {
        if utils::should_open(&options.input) {
            let data = utils::try_open_file(&options.input, options.debug);
            let mut interpreter = Interpreter::new(data);
            interpreter.run(&options);

        } else {
            warn!("Invalid input file extension. Accepted formats are .crapt and .crap");
            return;
        }
    } else {
        warn!("No input file given. Use -h or --help for help.");
        return;
    }

}
