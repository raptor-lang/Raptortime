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
    op_stack: Vec<i32>,
    memory: Vec<i32>,
    program_counter: usize,
    call_stack: Vec<StackFrame>
}

// Should this be here?
#[derive(Debug, Default)]
pub struct StackFrame {
    id: u32,
    locals: Vec<i32>,
    // The index of the first op in the op_stack that should be kept
    return_addr: usize
}

impl Interpreter {
    pub fn new(mut data: Vec<u8>) -> Interpreter {
        let header = read_header(&data);
        let const_table: ConstTable = read_const_table(data.drain(HEADER_SIZE..).collect::<Vec<u8>>().as_slice());
        let bytecode = data.drain(const_table.bc_counter..).collect();
        let mut i = Interpreter {
            header: header,
            const_table: const_table,
            bytecode: bytecode,
            op_stack: Vec::new(),
            memory: Vec::new(),
            program_counter: 0,
            call_stack: Vec::new()
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
                    self.op_stack.push($x);
                };
            }
            macro_rules! pop {
                () => {
                    self.op_stack.pop();
                };
            }
            macro_rules! operation {
                ($op:ident) => ({
                    let final_length: usize = self.op_stack.len().saturating_sub(2);
                    let val = self.op_stack.drain(final_length..).fold(
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

            macro_rules! push_frame {
                ($id:expr) => ({
                    let func_const = &self.const_table.funcs[$id as usize];
                    let mut sf = StackFrame {
                        id: $id,
                        locals: Vec::new(),
                        return_addr: 0
                    };
                    for _ in 0..func_const.arg_count {
                        sf.locals.push(pop!().unwrap());
                    }
                    sf.return_addr = self.program_counter;
                    sf.locals.resize((func_const.arg_count + func_const.local_count) as usize, 0);
                    self.call_stack.push(sf);
                });
            }
            // TODO: More macros, less code

            match instr {
                Instr::NOP => {},
                Instr::HALT => {
                    println!("HALT issued, stopped execution.");
                    if debug {
                        debug!("Stack: {:?}", self.op_stack);
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
                Instr::CALL => {
                    let id: u32 = self.get_next_4_bytes();
                    push_frame!(id);
                },
                Instr::RETURN => {}
                Instr::PRINT => {
                    println!("PRINT: {}", pop!().unwrap());
                },
                Instr::DUMP_STACK => {
                    println!("{:?}", self.op_stack);
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
