use std::{cell::RefCell, rc::Rc};

use crate::{circuit_input::CircuitInput, id_generator::IdGenerator, value_wire::ValueWire};

pub enum BoolData {
    Const(bool),
    Input(usize, Rc<CircuitInput>),
    And(usize, Rc<BoolWire>, Rc<BoolWire>),
    Inv(usize, Rc<BoolWire>), // Aka NOT
    Xor(usize, Rc<BoolWire>, Rc<BoolWire>),
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
            BoolData::Inv(id, _) => Some(*id),
            BoolData::Xor(id, _, _) => Some(*id),
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
            data: BoolData::Inv(id, BoolWire::and(&BoolWire::inv(a), &BoolWire::inv(b))),
        })
    }

    pub fn inv(a: &Rc<BoolWire>) -> Rc<BoolWire> {
        match &a.data {
            BoolData::Const(b) => {
                return Rc::new(BoolWire {
                    id_gen: a.id_gen.clone(),
                    data: BoolData::Const(!b),
                })
            }
            BoolData::Inv(_, a) => return a.clone(),
            _ => (),
        }

        BoolWire::inv_with_new_id(a)
    }

    pub fn inv_with_new_id(a: &Rc<BoolWire>) -> Rc<BoolWire> {
        let id = a.id_gen.borrow_mut().gen();

        Rc::new(BoolWire {
            id_gen: a.id_gen.clone(),
            data: BoolData::Inv(id, a.clone()),
        })
    }

    pub fn xor(a: &Rc<BoolWire>, b: &Rc<BoolWire>) -> Rc<BoolWire> {
        match &a.data {
            BoolData::Const(true) => return BoolWire::inv(b),
            BoolData::Const(false) => return b.clone(),
            _ => (),
        }

        match &b.data {
            BoolData::Const(true) => return BoolWire::inv(a),
            BoolData::Const(false) => return a.clone(),
            _ => (),
        }

        let id = a.id_gen.borrow_mut().gen();

        Rc::new(BoolWire {
            id_gen: a.id_gen.clone(),
            data: BoolData::Xor(id, a.clone(), b.clone()),
        })
    }

    pub fn copy_with_new_id(a: &Rc<BoolWire>) -> Rc<BoolWire> {
        if let BoolData::Inv(_, inv_a) = &a.data {
            return BoolWire::inv_with_new_id(inv_a);
        }

        BoolWire::inv_with_new_id(&BoolWire::inv_with_new_id(a))
    }
}
