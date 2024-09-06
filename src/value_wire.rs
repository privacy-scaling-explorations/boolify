use std::{cell::RefCell, rc::Rc};

use crate::{
    bool_wire::{BoolData, BoolWire},
    circuit_input::CircuitInput,
    id_generator::IdGenerator,
};

pub struct ValueWire {
    pub id_gen: Rc<RefCell<IdGenerator>>,
    pub bits: Vec<Rc<BoolWire>>,
}

impl ValueWire {
    pub fn new_input(name: String, size: usize, id_gen: &Rc<RefCell<IdGenerator>>) -> Self {
        let circuit_input = Rc::new(CircuitInput {
            name,
            id_start: id_gen.borrow_mut().peek(),
            size,
        });

        let mut bits = Vec::with_capacity(size);

        for _ in 0..size {
            bits.push(Rc::new(BoolWire {
                id_gen: id_gen.clone(),
                data: BoolData::Input(id_gen.borrow_mut().gen(), circuit_input.clone()),
            }));
        }

        // Reverse so that we have the least significant bit first but the ids put the most
        // significant bit first. This way we expose a big endian interface but can use little
        // endian internally which simplifies the code.
        bits.reverse();

        ValueWire {
            id_gen: id_gen.clone(),
            bits,
        }
    }

    pub fn at(&self, index: usize) -> Rc<BoolWire> {
        if index < self.bits.len() {
            self.bits[index].clone()
        } else {
            Rc::new(BoolWire {
                id_gen: self.id_gen.clone(),
                data: BoolData::Const(false),
            })
        }
    }

    pub fn add(a: &ValueWire, b: &ValueWire) -> ValueWire {
        let size = std::cmp::max(a.bits.len(), b.bits.len());
        let mut bits = Vec::with_capacity(size);

        let a_bit = a.at(0);
        let b_bit = b.at(0);

        bits.push(BoolWire::xor(&a_bit, &b_bit));
        let mut carry = BoolWire::and(&a_bit, &b_bit);

        for i in 1..size {
            let a_bit = a.at(i);
            let b_bit = b.at(i);

            let sum = BoolWire::xor(&a_bit, &b_bit);

            let new_carry = BoolWire::or(
                &BoolWire::and(&a_bit, &b_bit),
                &BoolWire::and(&carry, &BoolWire::xor(&a_bit, &b_bit)),
            );

            bits.push(BoolWire::xor(&sum, &carry));
            carry = new_carry;
        }

        ValueWire {
            id_gen: a.id_gen.clone(),
            bits,
        }
    }
}
