use std::{cell::RefCell, rc::Rc};

use crate::{
    bool_wire::{BoolData, BoolWire},
    circuit_input::CircuitInput,
    id_generator::IdGenerator,
};

#[derive(Clone)]
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

    pub fn new_const(mut value: u64, id_gen: &Rc<RefCell<IdGenerator>>) -> Self {
        let mut bits = Vec::new();

        while value > 0 {
            bits.push(Rc::new(BoolWire {
                id_gen: id_gen.clone(),
                data: BoolData::Const(value & 1 == 1),
            }));

            value >>= 1;
        }

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

            let new_carry =
                BoolWire::or(&BoolWire::and(&a_bit, &b_bit), &BoolWire::and(&carry, &sum));

            bits.push(BoolWire::xor(&sum, &carry));
            carry = new_carry;
        }

        ValueWire {
            id_gen: a.id_gen.clone(),
            bits,
        }
    }

    pub fn bit_not(a: &ValueWire) -> ValueWire {
        let bits = a.bits.iter().map(|bit| BoolWire::not(bit)).collect();

        ValueWire {
            id_gen: a.id_gen.clone(),
            bits,
        }
    }

    pub fn negate(&self) -> ValueWire {
        ValueWire::add(
            &ValueWire::bit_not(self),
            &ValueWire::new_const(1, &self.id_gen),
        )
    }

    pub fn resize(&self, size: usize) -> ValueWire {
        ValueWire {
            id_gen: self.id_gen.clone(),
            bits: (0..size).map(|i| self.at(i)).collect(),
        }
    }

    pub fn sub(a: &ValueWire, b: &ValueWire) -> ValueWire {
        let neg_b = if b.bits.len() < a.bits.len() {
            ValueWire::negate(&b.resize(a.bits.len()))
        } else {
            ValueWire::negate(b)
        };

        ValueWire::add(a, &neg_b)
    }

    pub fn shift_up_const(&self, amount: usize) -> ValueWire {
        let mut bits = Vec::with_capacity(self.bits.len());

        for _ in 0..amount {
            bits.push(Rc::new(BoolWire {
                id_gen: self.id_gen.clone(),
                data: BoolData::Const(false),
            }));
        }

        for i in 0..self.bits.len() - amount {
            bits.push(self.bits[i].clone());
        }

        ValueWire {
            id_gen: self.id_gen.clone(),
            bits,
        }
    }

    pub fn mul_bool(a: &Rc<BoolWire>, b: &ValueWire) -> ValueWire {
        let mut bits = Vec::with_capacity(b.bits.len());

        for i in 0..b.bits.len() {
            bits.push(BoolWire::and(a, &b.bits[i]));
        }

        ValueWire {
            id_gen: b.id_gen.clone(),
            bits,
        }
    }

    pub fn mul(a: &ValueWire, b: &ValueWire) -> ValueWire {
        let size = std::cmp::max(a.bits.len(), b.bits.len());

        let (sm, lg) = if a.bits.len() <= b.bits.len() {
            (a, b)
        } else {
            (b, a)
        };

        let mut sum_terms = Vec::new();

        for i in 0..sm.bits.len() {
            let term = ValueWire::mul_bool(&sm.bits[i], &lg.shift_up_const(i));
            sum_terms.push(term);
        }

        tree_sum(&sum_terms)
    }
}

fn tree_sum(values: &[ValueWire]) -> ValueWire {
    if values.len() == 1 {
        values[0].clone()
    } else {
        let mid = values.len() / 2;
        let left = tree_sum(&values[..mid]);
        let right = tree_sum(&values[mid..]);

        ValueWire::add(&left, &right)
    }
}
