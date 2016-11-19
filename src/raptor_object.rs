#![allow(non_camel_case_types)]
#![allow(dead_code)]

#[derive(Debug)]
pub enum RaptorType {
    NULL,
    INT,
    BOOL,
    FLOAT,
    USER_TYPE{id: u32}
}

#[derive(Debug)]
pub enum RaptorKind {
    VECTOR,
    OBJECT,
}

#[derive(Debug)]
pub struct RaptorObject {
    r_type: RaptorType,
    r_kind: RaptorKind,
    data: Vec<u32>,
}

impl RaptorObject {
    pub fn new() -> RaptorObject {
        RaptorObject {
            r_type: RaptorType::NULL,
            r_kind: RaptorKind::OBJECT,
            data: Vec::with_capacity(0),
        }
    }
}


