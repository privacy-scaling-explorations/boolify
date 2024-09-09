use std::{cell::RefCell, rc::Rc};

use crate::{circuit_input::CircuitInput, id_generator::IdGenerator, value_wire::ValueWire};

pub enum BoolData {
    Const(bool),
    Input(usize, Rc<CircuitInput>),
    And(usize, Rc<BoolWire>, Rc<BoolWire>),
    Or(usize, Rc<BoolWire>, Rc<BoolWire>),
    Not(usize, Rc<BoolWire>),
    Xor(usize, Rc<BoolWire>, Rc<BoolWire>),
    Copy(usize, Rc<BoolWire>),
}

pub struct BoolWire {
    pub id_gen: Rc<RefCell<IdGenerator>>,
    pub data: BoolData,
}

impl BoolWire {
    pub fn as_value(a: &Rc<BoolWire>) -> ValueWire {
        ValueWire {
            id_gen: a.id_gen.clone(),
            bits: vec![a.clone()],
        }
    }

    pub fn id(&self) -> Option<usize> {
        match &self.data {
            BoolData::Const(_) => None,
            BoolData::Input(id, _) => Some(*id),
            BoolData::And(id, _, _) => Some(*id),
            BoolData::Or(id, _, _) => Some(*id),
            BoolData::Not(id, _) => Some(*id),
            BoolData::Xor(id, _, _) => Some(*id),
            BoolData::Copy(id, _) => Some(*id),
        }
    }

    pub fn and(a: &Rc<BoolWire>, b: &Rc<BoolWire>) -> Rc<BoolWire> {
        match &a.data {
            BoolData::Const(false) => return a.clone(),
            BoolData::Const(true) => return b.clone(),
            _ => (),
        }

        match &b.data {
            BoolData::Const(false) => return b.clone(),
            BoolData::Const(true) => return a.clone(),
            _ => (),
        }

        let id = a.id_gen.borrow_mut().gen();

        Rc::new(BoolWire {
            id_gen: a.id_gen.clone(),
            data: BoolData::And(id, a.clone(), b.clone()),
        })
    }

    pub fn or(a: &Rc<BoolWire>, b: &Rc<BoolWire>) -> Rc<BoolWire> {
        match &a.data {
            BoolData::Const(true) => return a.clone(),
            BoolData::Const(false) => return b.clone(),
            _ => (),
        }

        match &b.data {
            BoolData::Const(true) => return b.clone(),
            BoolData::Const(false) => return a.clone(),
            _ => (),
        }

        let id = a.id_gen.borrow_mut().gen();

        Rc::new(BoolWire {
            id_gen: a.id_gen.clone(),
            data: BoolData::Or(id, a.clone(), b.clone()),
        })
    }

    pub fn not(a: &Rc<BoolWire>) -> Rc<BoolWire> {
        match &a.data {
            BoolData::Const(b) => {
                return Rc::new(BoolWire {
                    id_gen: a.id_gen.clone(),
                    data: BoolData::Const(!b),
                })
            }
            _ => (),
        }

        let id = a.id_gen.borrow_mut().gen();

        Rc::new(BoolWire {
            id_gen: a.id_gen.clone(),
            data: BoolData::Not(id, a.clone()),
        })
    }

    pub fn xor(a: &Rc<BoolWire>, b: &Rc<BoolWire>) -> Rc<BoolWire> {
        match &a.data {
            BoolData::Const(true) => return BoolWire::not(b),
            BoolData::Const(false) => return b.clone(),
            _ => (),
        }

        match &b.data {
            BoolData::Const(true) => return BoolWire::not(a),
            BoolData::Const(false) => return a.clone(),
            _ => (),
        }

        let id = a.id_gen.borrow_mut().gen();

        Rc::new(BoolWire {
            id_gen: a.id_gen.clone(),
            data: BoolData::Xor(id, a.clone(), b.clone()),
        })
    }

    pub fn copy(a: &Rc<BoolWire>) -> Rc<BoolWire> {
        let id = a.id_gen.borrow_mut().gen();

        Rc::new(BoolWire {
            id_gen: a.id_gen.clone(),
            data: BoolData::Copy(id, a.clone()),
        })
    }
}
