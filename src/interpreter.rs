use header::*;

#[derive(Debug, Default)]
pub struct Interpreter {
    // File data
    header: RaptorHeader,
    bytecode: Vec<u8>,
    
    // Runtime stuff
    stack: Vec<i32>,
    memory: Vec<i32>,
}

// TODO: Use more slices
impl Interpreter {
    pub fn new(mut data: Vec<u8>) -> Interpreter {
        let i = Interpreter {
            header: read_header(&data),
            bytecode: data.drain(HEADER_SIZE..).collect(),
            stack: Vec::new(),
            memory: Vec::new(),
        };

        i
    }

    pub fn run(&mut self) {
        debug!("Running...");
    }
}

// TODO: Add tests