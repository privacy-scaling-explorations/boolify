use std::{cell::RefCell, collections::HashMap, rc::Rc};

use boolify::{eval, generate_bristol, BoolWire, CircuitOutput, IdGenerator, ValueWire};

#[test]
fn test_2bit_add() {
    let id_gen = Rc::new(RefCell::new(IdGenerator::new()));

    let a = ValueWire::new_input("a", 2, &id_gen);
    let b = ValueWire::new_input("b", 2, &id_gen);

    let c = ValueWire::add(&a, &b);

    let outputs = vec![CircuitOutput::new("c", c)];

    let circuit = generate_bristol(&outputs);

    let bristol_string = circuit.get_bristol_string().unwrap();

    assert_eq!(
        bristol_string,
        vec![
            "4 8",
            "2 2 2",
            "1 2",
            "",
            "2 1 1 3 7 XOR",
            "2 1 0 2 4 XOR",
            "2 1 1 3 5 AND",
            "2 1 4 5 6 XOR",
            ""
        ]
        .join("\n")
    );
}

#[test]
fn test_8bit_xor_and_1() {
    let id_gen = Rc::new(RefCell::new(IdGenerator::new()));

    let a = ValueWire::new_input("a", 8, &id_gen);
    let b = ValueWire::new_input("b", 8, &id_gen);

    let c0 = ValueWire::add(&a, &b);
    let c = ValueWire::bit_and(&c0, &ValueWire::new_const(1, &id_gen));

    let outputs = vec![CircuitOutput::new("c", c)];

    let circuit = generate_bristol(&outputs);

    let bristol_string = circuit.get_bristol_string().unwrap();

    assert_eq!(
        bristol_string,
        vec![
            "8 24",
            "2 8 8",
            "1 8",
            "",
            "2 1 7 15 23 XOR",
            "2 1 0 0 16 XOR",
            "1 1 16 22 COPY",
            "1 1 16 21 COPY",
            "1 1 16 20 COPY",
            "1 1 16 19 COPY",
            "1 1 16 18 COPY",
            "1 1 16 17 COPY",
            "",
        ]
        .join("\n")
    );
}

#[test]
fn test_2bit_mul() {
    let id_gen = Rc::new(RefCell::new(IdGenerator::new()));

    let a = ValueWire::new_input("a", 2, &id_gen);
    let b = ValueWire::new_input("b", 2, &id_gen);

    let c = ValueWire::mul(&a, &b);

    let outputs = vec![CircuitOutput::new("c", c)];

    let circuit = generate_bristol(&outputs);

    let bristol_string = circuit.get_bristol_string().unwrap();

    assert_eq!(
        bristol_string,
        vec![
            "4 8",
            "2 2 2",
            "1 2",
            "",
            "2 1 1 3 7 AND",
            "2 1 1 2 4 AND",
            "2 1 0 3 5 AND",
            "2 1 4 5 6 XOR",
            ""
        ]
        .join("\n")
    );
}

#[test]
fn test_2bit_shl() {
    let id_gen = Rc::new(RefCell::new(IdGenerator::new()));

    let a = ValueWire::new_input("a", 2, &id_gen);
    let b = ValueWire::new_const(1, &id_gen);

    let c = ValueWire::bit_shl(&a, &b);

    let outputs = vec![CircuitOutput::new("c", c)];

    let circuit = generate_bristol(&outputs);

    let bristol_string = circuit.get_bristol_string().unwrap();

    assert_eq!(
        bristol_string,
        vec!["2 4", "1 2", "1 2", "", "2 1 0 0 3 XOR", "1 1 1 2 COPY", ""].join("\n")
    );
}

#[test]
fn test_2bit_shr() {
    let id_gen = Rc::new(RefCell::new(IdGenerator::new()));

    let a = ValueWire::new_input("a", 2, &id_gen);
    let b = ValueWire::new_const(1, &id_gen);

    let c = ValueWire::bit_shr(&a, &b);

    let outputs = vec![CircuitOutput::new("c", c)];

    let circuit = generate_bristol(&outputs);

    let bristol_string = circuit.get_bristol_string().unwrap();

    assert_eq!(
        bristol_string,
        vec!["2 4", "1 2", "1 2", "", "1 1 0 3 COPY", "2 1 0 0 2 XOR", ""].join("\n")
    );
}

#[test]
fn test_4bit_mul() {
    test_4bit_binary_op(ValueWire::mul, |a, b| (a * b) & 0xf);
}

#[test]
fn test_4bit_shl() {
    test_4bit_binary_op_with_const(ValueWire::bit_shl, |a, b| (a << b) & 0xf);
}

#[test]
fn test_4bit_shr() {
    test_4bit_binary_op_with_const(ValueWire::bit_shr, |a, b| (a >> b) & 0xf);
}

#[test]
fn test_4bit_div() {
    test_4bit_binary_op(ValueWire::div, |a, b| if b == 0 { 0xf } else { a / b });
}

#[test]
fn test_4bit_mod() {
    test_4bit_binary_op(ValueWire::mod_, |a, b| if b == 0 { a } else { a % b });
}

#[test]
fn test_4bit_add() {
    test_4bit_binary_op(ValueWire::add, |a, b| (a + b) & 0xf);
}

#[test]
fn test_4bit_sub() {
    test_4bit_binary_op(ValueWire::sub, |a, b| (a.wrapping_sub(b)) & 0xf);
}

#[test]
fn test_4bit_and() {
    test_4bit_binary_op(ValueWire::bit_and, |a, b| a & b);
}

#[test]
fn test_4bit_or() {
    test_4bit_binary_op(ValueWire::bit_or, |a, b| a | b);
}

#[test]
fn test_4bit_xor() {
    test_4bit_binary_op(ValueWire::bit_xor, |a, b| a ^ b);
}

#[test]
fn test_4bit_less_than() {
    test_4bit_binary_op(
        |a, b| BoolWire::as_value(&ValueWire::less_than(a, b)),
        |a, b| if a < b { 1 } else { 0 },
    );
}

#[test]
fn test_4bit_equal() {
    test_4bit_binary_op(
        |a, b| BoolWire::as_value(&ValueWire::equal(a, b)),
        |a, b| if a == b { 1 } else { 0 },
    );
}

#[test]
fn test_4bit_bool_and() {
    test_4bit_binary_op(
        |a, b| BoolWire::as_value(&ValueWire::bool_and(a, b)),
        |a, b| if a != 0 && b != 0 { 1 } else { 0 },
    );
}

#[test]
fn test_4bit_5mul() {
    test_4bit_unary_op(
        |a| ValueWire::mul(&ValueWire::new_const(5, &a.id_gen), a),
        |a| (5 * a) & 0xf,
    );
}

#[test]
fn test_4bit_1add() {
    test_4bit_unary_op(
        |a| ValueWire::add(&ValueWire::new_const(1, &a.id_gen), a),
        |a| (1 + a) & 0xf,
    );
}

#[test]
fn test_4bit_negate() {
    test_4bit_unary_op(ValueWire::negate, |a| (16 - a) & 0xf);
}

fn test_4bit_binary_op<F, G>(wire_op: F, op: G)
where
    F: Fn(&ValueWire, &ValueWire) -> ValueWire,
    G: Fn(usize, usize) -> usize,
{
    let id_gen = Rc::new(RefCell::new(IdGenerator::new()));

    let a = ValueWire::new_input("a", 4, &id_gen);
    let b = ValueWire::new_input("b", 4, &id_gen);

    let c = wire_op(&a, &b);

    let outputs = vec![CircuitOutput::new("c", c)];

    let circuit = generate_bristol(&outputs);

    for a in 0..16 {
        for b in 0..16 {
            let inputs = vec![("a", a), ("b", b)]
                .into_iter()
                .map(|(name, value)| (name.to_string(), value))
                .collect::<HashMap<String, usize>>();

            let result = eval(&circuit, &inputs);

            let expected = op(a, b);

            assert_eq!(result.get("c").unwrap(), &expected);
        }
    }
}

fn test_4bit_binary_op_with_const<F, G>(wire_op: F, op: G)
where
    F: Fn(&ValueWire, &ValueWire) -> ValueWire,
    G: Fn(usize, usize) -> usize,
{
    let id_gen = Rc::new(RefCell::new(IdGenerator::new()));

    let a = ValueWire::new_input("a", 4, &id_gen);
    let b = ValueWire::new_const(2, &id_gen);

    let c = wire_op(&a, &b);

    let outputs = vec![CircuitOutput::new("c", c)];

    let circuit = generate_bristol(&outputs);

    for a in 0..16 {
        let inputs = vec![("a", a), ("b", 2)]
            .into_iter()
            .map(|(name, value)| (name.to_string(), value))
            .collect::<HashMap<String, usize>>();

        let result = eval(&circuit, &inputs);

        let expected = op(a, b.as_usize().unwrap());

        assert_eq!(result.get("c").unwrap(), &expected);
    }
}

fn test_4bit_unary_op<F, G>(wire_op: F, op: G)
where
    F: Fn(&ValueWire) -> ValueWire,
    G: Fn(usize) -> usize,
{
    let id_gen = Rc::new(RefCell::new(IdGenerator::new()));

    let in_ = ValueWire::new_input("in", 4, &id_gen);
    let out = wire_op(&in_);

    let outputs = vec![CircuitOutput::new("out", out)];

    let circuit = generate_bristol(&outputs);

    for in_ in 0..16 {
        let inputs = vec![("in", in_)]
            .into_iter()
            .map(|(name, value)| (name.to_string(), value))
            .collect::<HashMap<String, usize>>();

        let result = eval(&circuit, &inputs);
        let expected = op(in_);

        assert_eq!(result.get("out").unwrap(), &expected);
    }
}
