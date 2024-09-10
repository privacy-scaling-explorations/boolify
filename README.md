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

2 1 3 7 21 XOR
2 1 2 6 8 XOR
2 1 3 7 9 AND
2 1 8 9 20 XOR
2 1 1 5 10 XOR
2 1 2 6 11 AND
2 1 9 8 12 AND
2 1 11 12 13 OR
2 1 10 13 19 XOR
2 1 0 4 14 XOR
2 1 1 5 15 AND
2 1 13 10 16 AND
2 1 15 16 17 OR
2 1 14 17 18 XOR
```

both circuits represent the addition of two numbers, but the arithmetic circuit simply uses one built-in addition gate, and the boolean circuit achieves (4-bit) addition using only boolean operations.

Bristol circuits are useful for doing MPC. One major category of MPC is garbled circuits, and these
require boolean circuits, hence this tool.

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
