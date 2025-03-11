# boolify

*Convert arithmetic circuits into boolean circuits*

## About

This library converts arithmetic [bristol fashion](https://nigelsmart.github.io/MPC-Circuits/) circuits like this:

```
1 3
2 1 1
1 1

2 1 0 1 2 AAdd
```

into *boolean* [bristol fashion](https://nigelsmart.github.io/MPC-Circuits/) circuits like this:

```
14 22
2 4 4
1 4

2 1 0 4 18 XOR
2 1 1 5 8 XOR
2 1 0 4 9 AND
2 1 8 9 19 XOR
2 1 2 6 10 XOR
2 1 1 5 11 AND
2 1 9 8 12 AND
2 1 11 12 13 XOR
2 1 10 13 20 XOR
2 1 3 7 14 XOR
2 1 2 6 15 AND
2 1 13 10 16 AND
2 1 15 16 17 XOR
2 1 14 17 21 XOR
```

both circuits represent the addition of two numbers, but the arithmetic circuit simply uses one built-in addition gate, and the boolean circuit achieves (4-bit) addition using only boolean operations.

Bristol circuits are useful for doing MPC. One major category of MPC is [garbled circuits](https://www.youtube.com/watch?v=FMZ-HARN0gI), and these require boolean circuits, hence this tool.

The following projects are useful for generating arithmetic circuits:
- [summon](https://github.com/voltrevo/summon) (write circuits in TypeScript)
- [circom-2-arithc](https://github.com/namnc/circom-2-arithc) (write circuits in circom)

The resulting boolean circuits can be useful in combination with:
- [MP-SPDZ](https://github.com/data61/MP-SPDZ)
- [mpz](https://github.com/privacy-scaling-explorations/mpz)
- [mpz-ts](https://github.com/voltrevo/mpz-ts)

## Quick Start / CLI

Create `input/circuit.txt` and `input/circuit_info.txt`.

`cargo run --bin boolify`

This will create `output/circuit.txt` and `output/circuit_info.txt`.

The CLI is currently hard-coded to use 16-bit arithmetic. You can change this in `src/cli.rs`.

## API

```rs
use boolify::boolify;
use bristol_circuit::BristolCircuit;

pub fn main() {
    let arith_circuit: BristolCircuit = todo!();
    let bits = 16; // or choose

    let bool_circuit = boolify(&arith_circuit, bits);
}
```

This strategy assumes a consistent bit width, which can be unnecessarily limiting.

You can go lower level and use the internal circuit model to combine operations with different sizes
like this:

```rs
use boolify::{generate_bristol, CircuitOutput, IdGenerator, ValueWire};

pub fn readme_demo() {
    let id_gen = IdGenerator::new_rc_refcell();

    let a = ValueWire::new_input("a", 100, &id_gen);
    let b = ValueWire::new_input("b", 80, &id_gen);

    // c is 100 bits, but requires fewer gates than a full 100x100-bit multiplier
    let c = ValueWire::mul(&a, &b);

    // Note this is a BoolWire rather than ValueWire, since `less than` results in a single bit of
    // information.
    let d = ValueWire::less_than(&c, &ValueWire::new_const(123, &id_gen));

    let outputs = vec![CircuitOutput::new("d", BoolWire::as_value(&d))];

    let bristol_circuit = generate_bristol(&outputs);

    println!("gates: {}", bristol_circuit.gates.len());
    // 28285
    // (multiplication of >64 bits is pretty heavy, modern ALUs require similar gate counts)
    // (use small bits if you can - eg a full 16x16-bit multiplier is only 648 gates)
}
```

Alternatively, you can get most of these benefits without using the internal circuit model by simply
using bit masking in your higher level language. For example, using summon with 32-bit circuit
generation, you can implement an 8-bit multiply-adder like this:

```ts
function mulAdd8Bit(a: number, b: number, c: number) {
    let res = a;

    res *= b;
    res &= 0xff;

    res += c;
    res &= 0xff;

    return res;
}
```

The inclusion of the bitmasking operations might feel like extra work (and it kinda is at compile-time), but it actually results in a smaller circuit because most of the bits are thrown away and all the circuitry required to generate them is discarded.
