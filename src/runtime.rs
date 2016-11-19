use interpreter::{Interpreter, StackFrame};
use raptor_object::RaptorObject;

#[derive(Debug, Default)]
pub struct Runtime {
    interpreter: Interpreter,
    call_stack: Vec<StackFrame>,
    options: ::Options,
    memory: Vec<RaptorObject>
}

impl Runtime {
    pub fn new(data: Vec<u8>, options: ::Options) -> Runtime {
        let mut r = Runtime {
            interpreter: Interpreter::new(data, options.debug),
            call_stack: Vec::new(),
            options: options,
            memory: vec![RaptorObject::new()],
        };
        let prog_bc = r.interpreter.prog_bytecode.clone();
        r.call_stack.push(
            StackFrame {
                bytecode: prog_bc,
                ..Default::default()
            });
        r
    }
    pub fn run(&mut self) {
        debug!("Running...");

        let debug = self.options.debug;

        while self.call_stack.len() > 0 {
            let dispatch_result = {
                let ln = self.call_stack.len();
                let mut last_frame = &mut self.call_stack[ln-1];
                last_frame.dispatch(&mut self.interpreter, debug)
            };
            // Push the new StackFrame, if CALL was issued
            match dispatch_result {
                None => {
                    debug!("Popped a frame. Current frame: {:?}",
                           self.call_stack[self.call_stack.len()-1]);
                    debug!("Op stack: {:?}", self.interpreter.op_stack);
                    self.call_stack.pop();
                },
                Some(frm) => {self.call_stack.push(frm);},
            }
        }
    }
}


