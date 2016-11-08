use header::*;
use constants::*;
use instructions::Instruction as Instr;
use num::FromPrimitive;

#[derive(Debug, Default)]
pub struct Interpreter {
    // File data
    header: RaptorHeader,
    bytecode: Vec<u8>,
    const_table: ConstTable,

    // Rutime stuff
    stack: Vec<i32>,
    memory: Vec<i32>,
    program_counter: usize,
}

impl Interpreter {
    pub fn new(mut data: Vec<u8>) -> Interpreter {
        let mut i = Interpreter {
            header: read_header(&data),
            const_table: read_const_table(&data),
            bytecode: data.drain(HEADER_SIZE..).collect(),
            stack: Vec::new(),
            memory: Vec::new(),
            program_counter: 0,
        };
        i.memory.resize(i.header.var_count as usize, 0);
        i
    }

    pub fn run(&mut self, options: &::Options) {
        use std::ops::*;
        use std::cmp::*;

        debug!("Running...");
        debug!("Bytecode: {:?}", self.bytecode);

        let debug = options.debug;
        
        // Main loop
        // Will break when .next() is None
        while self.program_counter != self.bytecode.len() {
            // info!("PC: {}", program_counter);

            // Use FromPrimitive trait to convert a value to its enum
            let instr = Instr::from_u8(self.bytecode[self.program_counter]);
            self.program_counter += 1;

            if instr.is_none() {
                warn!("Unimplemented instruction: {:04X}", self.bytecode[self.program_counter]);
                continue;
            }

            let instr = instr.unwrap();   // We're sure it's Some here, so unpack it.

            if options.debug {
                debug!("{:?}", instr);
            }

            macro_rules! push {
                ( $x:expr ) => {
                    self.stack.push($x);
                };
            }
            macro_rules! pop {
                () => {
                    self.stack.pop();
                };
            }
            macro_rules! operation {
                ($op:ident) => ({
                    let final_length: usize = self.stack.len().saturating_sub(2);
                    let val = self.stack.drain(final_length..).fold(
                        0, |acc, x| acc.$op(x)
                    );
                    push!(val);
                })
            }
            macro_rules! reljump {
                ($op:ident) => ({
                    let top = pop!().unwrap();
                    if top.$op(&0) {
                        reljump!();
                    } else {
                        self.get_next_4_bytes();
                        if debug {debug!("Jump not taken"); }
                    }
                });
                () => ({
                    let offset = (self.get_next_4_bytes() - 1) as i32;
                    // Need this if because you can't have negative usizes
                    if offset > 0 {
                        if debug {debug!("RELJUMP: {}", offset);}
                        self.program_counter += offset as usize;
                        if offset == 1 {debug!("RELJUMP 1 is redundant. This is a compiler bug")}
                    } else if offset < 0 {
                        if debug {debug!("RELJUMP: {}", offset);}
                        self.program_counter -= (-offset) as usize;
                    } else {
                        warn!("Invalid reljump offset: 0");
                    }
                });
            }
            // TODO: More macros, less code

            match instr {
                Instr::NOP => {},
                Instr::HALT => {
                    println!("HALT issued, stopped execution.");
                    if debug {
                        debug!("Stack: {:?}", self.stack);
                        debug!("Memory: {:?}", self.memory);
                    }
                },
                Instr::ICONST => {
                    let b = self.get_next_4_bytes() as i32;
                    push!(b);
                },
                Instr::POP => { pop!(); },
                Instr::ADD =>       { operation!(add);    },
                Instr::SUB =>       { operation!(sub);    },
                Instr::MULTIPLY =>  { operation!(mul);    },
                Instr::DIVIDE =>    { operation!(div);    },
                Instr::MODULUS =>   { operation!(rem);    },
                Instr::RSHIFT =>    { operation!(shl);    },
                Instr::LSHIFT =>    { operation!(shr);    },
                Instr::AND =>       { operation!(bitand); },
                Instr::OR =>        { operation!(bitor);  },
                Instr::NOT =>       {
                    let val = pop!().unwrap();
                    push!(val.not());
                },
                Instr::COMP => {
                    let a = pop!(); let b = pop!();
                    push!(if a > b {1} else if a < b {-1} else {0});
                },
                Instr::COMP_LT => {
                    let a = pop!(); let b = pop!();
                    push!(if a < b {1} else {0});
                },
                Instr::COMP_EQ => {
                    let a = pop!(); let b = pop!();
                    push!(if a == b {1} else {0});
                },
                Instr::COMP_GT => {
                    let a = pop!(); let b = pop!();
                    push!(if a > b {1} else {0});
                },
                Instr::RELJUMP => {reljump!();},
                Instr::RELJUMP_GT => {reljump!(gt);},
                Instr::RELJUMP_LT => {reljump!(lt);},
                Instr::RELJUMP_EQ => {reljump!(eq);},
                Instr::STORE => {
                    let index = self.get_next_4_bytes() as usize;
                    self.memory[index] = pop!().unwrap();
                },
                Instr::LOAD => {
                    let index = self.get_next_4_bytes() as usize;
                    push!(self.memory[index]);
                },
                Instr::CALL => {},
                Instr::RETURN => {}
                Instr::PRINT => {
                    println!("PRINT: {}", pop!().unwrap());
                },
                Instr::DUMP_STACK => {
                    println!("{:?}", self.stack);
                },
                Instr::DUMP_GLOBALS => {
                    println!("{:?}", self.memory);},
            }
        }
    }

    fn get_next_4_bytes(&mut self) -> u32 {
        let val = (self.bytecode[self.program_counter] as u32) << 24 |
                  (self.bytecode[self.program_counter + 1] as u32) << 16 |
                  (self.bytecode[self.program_counter + 2] as u32) << 8 |
                  (self.bytecode[self.program_counter + 3] as u32);
        self.program_counter += 4;
        debug!("get_next_4_bytes: 0x{:04X}", val);
        val
    }
}

// TODO: Add tests
