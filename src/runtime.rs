use std::{cell::RefCell, rc::Rc};

pub type Ref<T> = Rc<T>;
pub type Mut<T> = Rc<RefCell<T>>;

#[derive(Debug, PartialEq, Clone)]
pub struct Fn {
    bytecode: Ref<[Op]>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct Bytes(Ref<[u8]>);

#[derive(Debug, PartialEq, Clone)]
pub struct Int(i128);

#[derive(Debug, PartialEq, Clone)]
pub struct Float(f64);

#[derive(Debug, PartialEq, Clone)]
pub struct Str(Ref<str>);

#[derive(Debug, PartialEq, Clone)]
pub struct Tuple(Ref<[Variant]>);

#[derive(Debug, PartialEq, Clone)]
pub struct Array(Mut<[Variant]>);

#[derive(Debug, PartialEq, Clone)]
pub struct Struct(Mut<StructInstance>);

#[derive(Debug, PartialEq, Clone)]
pub struct StructInstance {
    fields: Ref<Type>,
    values: Box<[Variant]>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct Type {
    field: Box<[Field]>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct Field {
    name: String,
}

#[derive(Debug, PartialEq, Clone)]
pub enum Variant {
    Fn(Fn),
    Int(Int),
    Float(Int),
    BigInt(Bytes),
    Bytes(Bytes),
    Str(Str),
    Array(Array),
    Struct(Struct),
}

#[derive(Debug, PartialEq, Clone)]
#[repr(u8)]
pub enum Op {
    Add,
    Sub,
    Mul,
    Div,
    Shl,
    Shr,
    Shra,
    Index,
    Dot,
    Call,
    Ok,
    Err,
    Fn,
    Int,
    Bytes,
    Str,
    Array,
    Tuple,
    Map,
    Loc,
}
