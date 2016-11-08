use std::fmt;
use num::FromPrimitive;

#[derive(Debug)]
pub struct FuncConst {
    arg_count: u32,
    local_count: u32,
    body: Vec<u8>
}

#[derive(Default)]
pub struct ConstTable {
    funcs: Vec<FuncConst>
    // TODO: Add more types
}

impl fmt::Debug for ConstTable {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        //TODO
        unimplemented!()
    }
}


pub fn read_const_table(data: &[u8]) -> ConstTable {

    let mut bc_counter = 0;
    let mut const_table: ConstTable = Default::default();

    macro_rules! get_next_4_bytes {
        () => {{
            let val = (data[bc_counter] as u32) << 24 |
            (data[bc_counter + 1] as u32) << 16 |
            (data[bc_counter + 2] as u32) << 8 |
            (data[bc_counter + 3] as u32);
            bc_counter += 4;
            debug!("get_next_4_bytes: 0x{:04X}", val);
            val
        }}
    }

    while bc_counter != data.len() {

        let instr = ConstInstr::from_u8(data[bc_counter]);
        bc_counter += 1;

        if instr.is_none() {
            warn!("Unimplemented constants table instruction: {:04X}", data[bc_counter]);
            continue;
        }

        let instr = instr.unwrap();

        match instr {
            ConstInstr::FUNC => {
                const_table.funcs[get_next_4_bytes!() as usize] = FuncConst {
                    arg_count: get_next_4_bytes!(),
                    local_count: get_next_4_bytes!(),
                    body: data[bc_counter..(bc_counter+get_next_4_bytes!() as usize)].to_vec()
                };
                debug!("Added a function to the constants table")
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
