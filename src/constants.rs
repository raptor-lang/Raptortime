use std::str;
use num::FromPrimitive;

#[derive(Debug, Default, Clone)]
pub struct FuncConst {
    pub name: String,
    pub arg_count: u32,
    pub local_count: u32,
    pub body: Vec<u8>
}

#[derive(Debug, Default)]
pub struct ConstTable {
    pub funcs: Vec<FuncConst>,
    pub bc_counter: usize
}

// Eats a null byte terminated string
fn eat_string(data: &[u8], const_table: &mut ConstTable) -> String {
    use std::ffi::CString;

    let str_start = const_table.bc_counter;
    let mut str_len: usize = 0;
    while data[str_start + str_len] != 0x00 {
        str_len += 1;
    }
    let string = CString::new(&data[str_start..(str_start+str_len)]).unwrap();
    const_table.bc_counter += str_len + 1; // + 1 for null byte
    //debug!("Ate string {:?} of length {}", string, str_len);
    string.into_string().unwrap()
}

#[inline]
fn get_next_4_bytes(data: &[u8], const_table: &mut ConstTable) -> u32 {

    let val = (data[const_table.bc_counter] as u32) << 24 |
        (data[const_table.bc_counter + 1] as u32) << 16 |
        (data[const_table.bc_counter + 2] as u32) << 8 |
        (data[const_table.bc_counter + 3] as u32);
    const_table.bc_counter += 4;
    debug!("get_next_4_bytes: 0x{:04X}", val);
    val
}

pub fn read_const_table(data: &[u8]) -> ConstTable {

    let mut const_table: ConstTable = Default::default();

    macro_rules! get_next_4_bytes {
        () => (get_next_4_bytes(&data, &mut const_table))
    }

    while const_table.bc_counter != data.len() {
        let instr = ConstInstr::from_u8(data[const_table.bc_counter]);

        const_table.bc_counter += 1;
        if instr.is_none() {
            warn!("Unimplemented constants table instruction: {:04X}",
                  data[const_table.bc_counter - 1]);
            continue;
        }

        let instr = instr.unwrap();

        match instr {
            ConstInstr::FUNC => {
                // TODO: use this id
                let id = get_next_4_bytes!() as usize;
                let name = eat_string(&data, &mut const_table);
                let arg_count = get_next_4_bytes!();
                let local_count = get_next_4_bytes!();
                let bc_length = get_next_4_bytes!() as usize;
                let body = data[const_table.bc_counter..{
                    const_table.bc_counter += bc_length;
                    const_table.bc_counter + 1
                }].to_vec();

                if id >= const_table.funcs.len() {
                    const_table.funcs.resize(id + 1,FuncConst{
                        ..Default::default()
                    })
                }

                info!("Added function \"{}\" to the constants table", name);
                const_table.funcs[id] = FuncConst {
                    name: name,
                    arg_count: arg_count,
                    local_count: local_count,
                    body: body
                };
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
