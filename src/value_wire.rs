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
    pub fn new_input(name: &str, size: usize, id_gen: &Rc<RefCell<IdGenerator>>) -> Self {
        let circuit_input = Rc::new(CircuitInput {
            name: name.to_string(),
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

    pub fn new_const_f64(value: f64, id_gen: &Rc<RefCell<IdGenerator>>) -> Self {
        let mut bits = Vec::new();

        let max_safe_int = 9007199254740991.0;
        assert!(value >= 0.0 && value == value.trunc() && value <= max_safe_int);

        let mut value = value as u64;

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

    pub fn as_usize(&self) -> Option<usize> {
        if self.bits.len() > (usize::BITS as usize) {
            return None;
        }

        let mut value = 0;

        for i in 0..self.bits.len() {
            let BoolData::Const(bit) = self.bits[i].data else {
                return None;
            };

            value |= (bit as usize) << i;
        }

        Some(value)
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
                BoolWire::xor(&BoolWire::and(&a_bit, &b_bit), &BoolWire::and(&carry, &sum));

            bits.push(BoolWire::xor(&sum, &carry));
            carry = new_carry;
        }

        ValueWire {
            id_gen: a.id_gen.clone(),
            bits,
        }
    }

    pub fn bit_not(a: &ValueWire) -> ValueWire {
        let bits = a.bits.iter().map(|bit| BoolWire::inv(bit)).collect();

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
        if amount >= self.bits.len() {
            return ValueWire::new_const(0, &self.id_gen);
        }

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

    pub fn shift_down_const(&self, amount: usize) -> ValueWire {
        if amount >= self.bits.len() {
            return ValueWire::new_const(0, &self.id_gen);
        }

        let mut bits = Vec::with_capacity(self.bits.len());

        for i in amount..self.bits.len() {
            bits.push(self.bits[i].clone());
        }

        for _ in 0..amount {
            bits.push(Rc::new(BoolWire {
                id_gen: self.id_gen.clone(),
                data: BoolData::Const(false),
            }));
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
        let (sm, lg) = if a.bits.len() <= b.bits.len() {
            (a, b)
        } else {
            (b, a)
        };

        let mut sum_terms = Vec::new();

        for i in 0..sm.bits.len() {
            if let BoolData::Const(false) = sm.bits[i].data {
                continue;
            }

            let term = ValueWire::mul_bool(&sm.bits[i], &lg.shift_up_const(i));
            sum_terms.push(term);
        }

        tree_sum(&sum_terms, &a.id_gen)
    }

    pub fn exp(a: &ValueWire, b: &ValueWire) -> ValueWire {
        match b.as_usize() {
            Some(n) => {
                if n == 0 {
                    // Base case: any number to the power of 0 is 1.
                    return ValueWire::new_const(1, &a.id_gen);
                } else if n == 1 {
                    return a.clone();
                } else if n % 2 == 0 {
                    // If n is even, compute (a * a)^(n / 2).
                    let half = ValueWire::mul(a, a);

                    return ValueWire::exp(&half, &ValueWire::new_const(n / 2, &a.id_gen));
                } else {
                    // If n is odd: compute a * (a * a)^((n - 1) / 2).
                    let half = ValueWire::mul(a, a);
                    let reduced =
                        ValueWire::exp(&half, &ValueWire::new_const((n - 1) / 2, &a.id_gen));

                    return ValueWire::mul(a, &reduced);
                }
            }
            None => panic!("Wire 'b' is not a constant"),
        }
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
                BoolWire::inv(&BoolWire::xor(&a.at(0), &b.at(0))),
                BoolWire::and(&BoolWire::inv(&a.at(0)), &b.at(0)),
            );
        }

        let (a0, a1) = a.split_at(size / 2);
        let (b0, b1) = b.split_at(size / 2);

        let (eq0, lt0) = ValueWire::cmp(&a0, &b0);
        let (eq1, lt1) = ValueWire::cmp(&a1, &b1);

        let eq = BoolWire::and(&eq0, &eq1);
        let lt = BoolWire::xor(&lt1, &BoolWire::and(&eq1, &lt0));

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
        BoolWire::inv(&ValueWire::greater_than(a, b))
    }

    pub fn greater_than_or_eq(a: &ValueWire, b: &ValueWire) -> Rc<BoolWire> {
        BoolWire::inv(&ValueWire::less_than(a, b))
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
            return BoolWire::inv(&BoolWire::xor(&a.at(0), &b.at(0)));
        }

        let (a0, a1) = a.split_at(size / 2);
        let (b0, b1) = b.split_at(size / 2);

        let eq0 = ValueWire::equal(&a0, &b0);
        let eq1 = ValueWire::equal(&a1, &b1);

        BoolWire::and(&eq0, &eq1)
    }

    pub fn not_equal(a: &ValueWire, b: &ValueWire) -> Rc<BoolWire> {
        BoolWire::inv(&ValueWire::equal(a, b))
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
        BoolWire::inv(&a.to_bool())
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

    pub fn bit_shl(a: &ValueWire, b: &ValueWire) -> ValueWire {
        match b.as_usize() {
            Some(n) => a.shift_up_const(n),
            None => panic!("Wire 'b' is not a constant"),
        }
    }

    pub fn bit_shr(a: &ValueWire, b: &ValueWire) -> ValueWire {
        match b.as_usize() {
            Some(n) => a.shift_down_const(n),
            None => panic!("Wire 'b' is not a constant"),
        }
    }

    pub fn quotient_remainder(a: &ValueWire, b: &ValueWire) -> (ValueWire, ValueWire) {
        let size = std::cmp::max(a.bits.len(), b.bits.len());
        let a = a.resize(size);
        let b = b.resize(size);

        let mut shifts_valid = Vec::<Rc<BoolWire>>::new();
        shifts_valid.push(Rc::new(BoolWire {
            id_gen: a.id_gen.clone(),
            data: BoolData::Const(true),
        }));

        for i in 1..size {
            shifts_valid.push(BoolWire::and(
                &shifts_valid[i - 1],
                &BoolWire::inv(&b.at(size - i)),
            ));
        }

        let mut quotient = ValueWire::new_const(0, &a.id_gen).resize(size);
        let mut rem = a.clone();

        for i in (0..size).rev() {
            let valid = &shifts_valid[i];
            let shift_b = b.shift_up_const(i);
            let less_than = ValueWire::less_than_or_eq(&shift_b, &rem);

            let apply = BoolWire::and(valid, &less_than);
            let apply_rem = ValueWire::sub(&rem, &shift_b);

            quotient.bits[i] = apply.clone();

            rem = ValueWire::bit_xor(
                &ValueWire::mul_bool(&apply, &apply_rem),
                &ValueWire::mul_bool(&BoolWire::inv(&apply), &rem),
            );
        }

        (quotient, rem)
    }

    pub fn div(a: &ValueWire, b: &ValueWire) -> ValueWire {
        ValueWire::quotient_remainder(a, b).0
    }

    pub fn mod_(a: &ValueWire, b: &ValueWire) -> ValueWire {
        ValueWire::quotient_remainder(a, b).1
    }
}

fn tree_sum(values: &[ValueWire], id_gen: &Rc<RefCell<IdGenerator>>) -> ValueWire {
    if values.len() == 0 {
        ValueWire::new_const(0, id_gen)
    } else if values.len() == 1 {
        values[0].clone()
    } else {
        let mid = values.len() / 2;
        let left = tree_sum(&values[..mid], id_gen);
        let right = tree_sum(&values[mid..], id_gen);

        ValueWire::add(&left, &right)
    }
}
