use std::fmt;
use num::FromPrimitive;

#[derive(Debug)]
pub struct FuncConst {
    pub arg_count: u32,
    pub local_count: u32,
    pub body: Vec<u8>
}

#[derive(Default)]
pub struct ConstTable {
    pub funcs: Vec<FuncConst>,
    pub bc_counter: usize
}

impl fmt::Debug for ConstTable {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        //TODO
        unimplemented!()
    }
}


pub fn read_const_table(data: &[u8]) -> ConstTable {

    let mut const_table: ConstTable = Default::default();

    macro_rules! get_next_4_bytes {
        () => {{
            let val = (data[const_table.bc_counter] as u32) << 24 |
            (data[const_table.bc_counter + 1] as u32) << 16 |
            (data[const_table.bc_counter + 2] as u32) << 8 |
            (data[const_table.bc_counter + 3] as u32);
            const_table.bc_counter += 4;
            debug!("get_next_4_bytes: 0x{:04X}", val);
            val
        }}
    }

    while const_table.bc_counter != data.len() {

        let instr = ConstInstr::from_u8(data[const_table.bc_counter]);

        const_table.bc_counter += 1;
        if instr.is_none() {
            warn!("Unimplemented constants table instruction: {:04X}", data[const_table.bc_counter - 1]);
            continue;
        }

        let instr = instr.unwrap();

        match instr {
            ConstInstr::FUNC => {
                // TODO: remove this on the compiler end
                get_next_4_bytes!();
                let arg_count = get_next_4_bytes!();
                let local_count = get_next_4_bytes!();
                let bc_length = get_next_4_bytes!() as usize;
                const_table.funcs.push(FuncConst {
                    arg_count: arg_count,
                    local_count: local_count,
                    body: data[const_table.bc_counter..{const_table.bc_counter += bc_length; const_table.bc_counter + 1}].to_vec()
                });
                debug!("Added a function to the constants table");
            },
            ConstInstr::END => {
                debug!("Reached end of constants table");
                break;
            }
        }
    }
    const_table
}


enum_from_primitive! {
    #[allow(non_camel_case_types)]
    #[derive(Debug, PartialEq)]
    pub enum ConstInstr {
        FUNC = 0xF0,
        END = 0xED
    }
}
