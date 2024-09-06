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

    pub fn new_const(mut value: usize, id_gen: &Rc<RefCell<IdGenerator>>) -> Self {
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
        if size == self.bits.len() {
            return self.clone();
        }

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

    fn split_at(&self, split_point: usize) -> (ValueWire, ValueWire) {
        if self.bits.len() <= split_point {
            return (self.clone(), ValueWire::new_const(0, &self.id_gen));
        }

        let left = ValueWire {
            id_gen: self.id_gen.clone(),
            bits: self.bits[..split_point].to_vec(),
        };

        let right = ValueWire {
            id_gen: self.id_gen.clone(),
            bits: self.bits[split_point..].to_vec(),
        };

        (left, right)
    }

    // eq, lt
    fn cmp(a: &ValueWire, b: &ValueWire) -> (Rc<BoolWire>, Rc<BoolWire>) {
        let size = std::cmp::max(a.bits.len(), b.bits.len());

        if size == 0 {
            return (
                Rc::new(BoolWire {
                    id_gen: a.id_gen.clone(),
                    data: BoolData::Const(false),
                }),
                Rc::new(BoolWire {
                    id_gen: a.id_gen.clone(),
                    data: BoolData::Const(false),
                }),
            );
        }

        if size == 1 {
            return (
                BoolWire::not(&BoolWire::xor(&a.at(0), &b.at(0))),
                BoolWire::and(&BoolWire::not(&a.at(0)), &b.at(0)),
            );
        }

        let (a0, a1) = a.split_at(size / 2);
        let (b0, b1) = b.split_at(size / 2);

        let (eq0, lt0) = ValueWire::cmp(&a0, &b0);
        let (eq1, lt1) = ValueWire::cmp(&a1, &b1);

        let eq = BoolWire::and(&eq0, &eq1);
        let lt = BoolWire::or(&lt1, &BoolWire::and(&eq1, &lt0));

        (eq, lt)
    }

    pub fn less_than(a: &ValueWire, b: &ValueWire) -> Rc<BoolWire> {
        let (_eq, lt) = ValueWire::cmp(a, b);

        lt
    }

    pub fn greater_than(a: &ValueWire, b: &ValueWire) -> Rc<BoolWire> {
        ValueWire::less_than(b, a)
    }

    pub fn less_than_or_eq(a: &ValueWire, b: &ValueWire) -> Rc<BoolWire> {
        BoolWire::not(&ValueWire::greater_than(a, b))
    }

    pub fn greater_than_or_eq(a: &ValueWire, b: &ValueWire) -> Rc<BoolWire> {
        BoolWire::not(&ValueWire::less_than(a, b))
    }

    pub fn equal(a: &ValueWire, b: &ValueWire) -> Rc<BoolWire> {
        let size = std::cmp::max(a.bits.len(), b.bits.len());

        if size == 0 {
            return Rc::new(BoolWire {
                id_gen: a.id_gen.clone(),
                data: BoolData::Const(true),
            });
        }

        if size == 1 {
            return BoolWire::not(&BoolWire::xor(&a.at(0), &b.at(0)));
        }

        let (a0, a1) = a.split_at(size / 2);
        let (b0, b1) = b.split_at(size / 2);

        let eq0 = ValueWire::equal(&a0, &b0);
        let eq1 = ValueWire::equal(&a1, &b1);

        BoolWire::and(&eq0, &eq1)
    }

    pub fn not_equal(a: &ValueWire, b: &ValueWire) -> Rc<BoolWire> {
        BoolWire::not(&ValueWire::equal(a, b))
    }

    pub fn to_bool(&self) -> Rc<BoolWire> {
        if self.bits.len() == 0 {
            return Rc::new(BoolWire {
                id_gen: self.id_gen.clone(),
                data: BoolData::Const(false),
            });
        }

        if self.bits.len() == 1 {
            return self.bits[0].clone();
        }

        let (left, right) = self.split_at(self.bits.len() / 2);

        BoolWire::or(&left.to_bool(), &right.to_bool())
    }

    pub fn bool_and(a: &ValueWire, b: &ValueWire) -> Rc<BoolWire> {
        BoolWire::and(&a.to_bool(), &b.to_bool())
    }

    pub fn bool_or(a: &ValueWire, b: &ValueWire) -> Rc<BoolWire> {
        BoolWire::or(&a.to_bool(), &b.to_bool())
    }

    pub fn bool_not(a: &ValueWire) -> Rc<BoolWire> {
        BoolWire::not(&a.to_bool())
    }

    pub fn bool_xor(a: &ValueWire, b: &ValueWire) -> Rc<BoolWire> {
        BoolWire::xor(&a.to_bool(), &b.to_bool())
    }

    pub fn bit_and(a: &ValueWire, b: &ValueWire) -> ValueWire {
        let size = std::cmp::max(a.bits.len(), b.bits.len());

        ValueWire {
            id_gen: a.id_gen.clone(),
            bits: (0..size)
                .map(|i| BoolWire::and(&a.at(i), &b.at(i)))
                .collect(),
        }
    }

    pub fn bit_or(a: &ValueWire, b: &ValueWire) -> ValueWire {
        let size = std::cmp::max(a.bits.len(), b.bits.len());

        ValueWire {
            id_gen: a.id_gen.clone(),
            bits: (0..size)
                .map(|i| BoolWire::or(&a.at(i), &b.at(i)))
                .collect(),
        }
    }

    pub fn bit_xor(a: &ValueWire, b: &ValueWire) -> ValueWire {
        let size = std::cmp::max(a.bits.len(), b.bits.len());

        ValueWire {
            id_gen: a.id_gen.clone(),
            bits: (0..size)
                .map(|i| BoolWire::xor(&a.at(i), &b.at(i)))
                .collect(),
        }
    }
}

fn tree_sum(values: &[ValueWire]) -> ValueWire {
    if values.len() == 0 {
        ValueWire::new_const(0, &values[0].id_gen)
    } else if values.len() == 1 {
        values[0].clone()
    } else {
        let mid = values.len() / 2;
        let left = tree_sum(&values[..mid]);
        let right = tree_sum(&values[mid..]);

        ValueWire::add(&left, &right)
    }
}
