use header::*;
use constants::*;
use instructions::Instruction as Instr;
use num::FromPrimitive;
use std::fmt;

#[derive(Debug, Default)]
pub struct Runtime {
    interpreter: Interpreter,
    call_stack: Vec<StackFrame>,
}

impl Runtime {
    pub fn new(mut data: Vec<u8>) -> Runtime {
        let mut r = Runtime {
            interpreter: Interpreter::new(data),
            call_stack: Vec::new(),
        };
        r
    }
    pub fn run(&mut self, options: &::Options) {
        debug!("Running...");

        let debug = options.debug;

        loop {
            // Need to wrap this here to limit the scope of last_frame borrow
            let dispatch_result = {
                let ln = self.call_stack.len();
                let mut last_frame = &mut self.call_stack[ln-1];
                last_frame.dispatch(&mut self.interpreter, debug)
            };
            // Push the new StackFrame, if CALL was issued
            if let Some(frm) = dispatch_result {
                self.call_stack.push(frm);
            }
        }
    }
}

#[derive(Debug, Default)]
pub struct Interpreter {
    // File data
    header: RaptorHeader,
    const_table: ConstTable,

    // Rutime stuff
    op_stack: Vec<i32>,
    memory: Vec<i32>,
}

// Should this be here?

#[derive(Debug, Default, Clone)]
pub struct StackFrame {
    locals: Vec<i32>,
    // The index of the first op in the op_stack that should be kept
    return_addr: usize,
    bytecode: Vec<u8>,
    bc_counter: usize
}

impl Interpreter {
    pub fn new(mut data: Vec<u8>) -> Interpreter {
        debug!("Bytecode length: {} bytes", data.len());
        let header = read_header(&data);
        data.drain(..HEADER_SIZE);
        let const_table: ConstTable = read_const_table(data.as_slice());
        debug!("Constant table length: {} bytes", const_table.bc_counter);
        debug!("Bytecode length: {} bytes", data.len());
        data.drain(..const_table.bc_counter);
        let sf = StackFrame {
            bytecode: data,
            ..Default::default()
        };
        let mut i = Interpreter {
            header: header,
            const_table: const_table,
            op_stack: Vec::new(),
            memory: Vec::new(),
        };
        i.memory.resize(i.header.var_count as usize, 0);
        i
    }
}

impl StackFrame {

    fn get_next_4_bytes(&mut self) -> u32 {
        let val = (self.bytecode[self.bc_counter] as u32) << 24 |
        (self.bytecode[self.bc_counter + 1] as u32) << 16 |
        (self.bytecode[self.bc_counter + 2] as u32) << 8 |
        (self.bytecode[self.bc_counter + 3] as u32);
        self.bc_counter += 4;
        debug!("get_next_4_bytes: 0x{:04X}", val);
        val
    }


    fn dispatch(&mut self, inpr: &mut Interpreter, debug: bool) -> Option<StackFrame> {
        use std::ops::*;
        use std::cmp::*;

        // Main loop
        while self.bc_counter != self.bytecode.len() {
            // info!("PC: {}", bc_counter);

            // Use FromPrimitive trait to convert a value to its enum
            let instr = Instr::from_u8(self.bytecode[self.bc_counter]);
            self.bc_counter += 1;

            if instr.is_none() {
                warn!("Unimplemented instruction: {:04X}", self.bytecode[self.bc_counter]);
                continue;
            }

            let instr = instr.unwrap();   // We're sure it's Some here, so unpack it.

            if debug {
                debug!("{:?}", instr);
            }

            macro_rules! push {
                ( $x:expr ) => {
                    inpr.op_stack.push($x);
                };
            }
            macro_rules! pop {
                () => {
                    inpr.op_stack.pop();
                };
            }
            macro_rules! operation {
                ($op:ident) => ({
                    let final_length: usize = inpr.op_stack.len().saturating_sub(2);
                    let val = inpr.op_stack.drain(final_length..).fold(
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
                        self.bc_counter += offset as usize;
                        if offset == 1 {debug!("RELJUMP 1 is redundant. This is a compiler bug")}
                    } else if offset < 0 {
                        if debug {debug!("RELJUMP: {}", offset);}
                        self.bc_counter -= (-offset) as usize;
                    } else {
                        warn!("Invalid reljump offset: 0");
                    }
                });
            }

            macro_rules! push_frame {
                ($id:expr) => ({
                    let func_const = &inpr.const_table.funcs[$id as usize];
                    let mut sf = StackFrame {
                        locals: Vec::new(),
                        ..Default::default()
                    };
                    for _ in 0..func_const.arg_count {
                        sf.locals.push(pop!().unwrap());
                    }
                    sf.return_addr = inpr.op_stack.len();
                    sf.locals.resize((func_const.arg_count + func_const.local_count) as usize, 0);
                    if debug {
                        debug!("Pushed new frame: {}    {:?}", $id, sf);
                        debug!("Op stack: {:?}", inpr.op_stack);
                    }
                    return Some(sf);
                });
            }

            match instr {
                Instr::NOP => {},
                Instr::HALT => {
                    println!("HALT issued, stopped execution.");
                    if debug {
                        debug!("Stack: {:?}", inpr.op_stack);
                        debug!("Memory: {:?}", inpr.memory);
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
                    self.locals[index] = pop!().unwrap();
                },
                Instr::LOAD => {
                    let index = self.get_next_4_bytes() as usize;
                    push!(self.locals[index]);
                },
                Instr::CALL => {
                    let id: u32 = self.get_next_4_bytes();
                    push_frame!(id);
                },
                Instr::RETURN => {
                    let val = pop!();
                    inpr.op_stack.resize(self.return_addr, 0)
                }
                Instr::PRINT => {
                    println!("PRINT: {}", pop!().unwrap());
                },
                Instr::DUMP_STACK => {
                    println!("{:?}", inpr.op_stack);
                },
                Instr::DUMP_GLOBALS => {
                    println!("{:?}", inpr.memory);},
            }
        }
        return None;    // Pop the current frame
    }
}

// TODO: Add tests
