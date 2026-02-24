# Pinocchio Solana program template

This is a template for optimized Solana program development using the Pinocchio framework. This template was built by [Exo Technologies](https://exotechnologies.xyz).

## Build

From project root

```
cargo build-sbf
```

## Generating IDL

This repository uses Shank for IDL generation.

Install the Shank CLI

```
cargo install shank-cli
```

Generate IDL

```
shank idl -r program -o idl
// OR
yarn generate-idl
```

## Generating Clients

_Ensure the IDL has been created or updated using the above IDL generation steps._

Install dependencies

```
yarn install
```

Run client generation script

```
yarn generate:clients
```

## Running Tests

Integration tests are written using [LiteSvm](https://github.com/LiteSVM/litesvm). To run integration tests, from project root build and then run

```
cargo test
```
https://github.com/MeteoraAg/dlmm-sdk/blob/main/ts-client/src/examples 

Pattern A: Router Both Swaps
[Router]
  ├─> Swap A (CPI)
  │     └─> TokenProgram x2
  └─> Swap B (CPI)
        └─> TokenProgram x2

Pattern B: Introspect + 1 CPI
[Swap A top-level]   <-- no nested invoke
  └─> TokenProgram x2
[Router]
  └─> Swap B (CPI)
        └─> TokenProgram x2



//ansi_term , cipher - duplicates in Cargo.lock from the github.com/exo-tech-xyz/pinocchio-project need to make PR

//then fix base64ct build error by downgrading it https://solana.stackexchange.com/questions/23578/how-to-build-with-solana-program-3-0-0-cargo-build-sbf-3-0-6-fails-with-edition


Run in terminal to get/verify hex 
```python
python3 -c "
import base64, struct

def b58decode(s):
    alphabet = '123456789ABCDEFGHJKLMNPQRSTUVWXYZabcdefghijkmnopqrstuvwxyz'
    n = 0
    for c in s:
        n = n * 58 + alphabet.index(c)
    return n.to_bytes(32, 'big')

key = b58decode('LBUZKhRxPF3XUpBCjp4YzTKgLccjZhTSDM9YuVaPwxo')
print(', '.join(f'0x{b:02X}' for b in key))
"
```


Also for discriminators
```
python3 -c "
import hashlib
h = hashlib.sha256(b'global:swap2').digest()
print('Expected: ', ', '.join(f'0x{b:02X}' for b in h[:8]))
print('Got:      ', '0x41, 0x4B, 0x3F, 0x4C, 0xEB, 0x5B, 0x5B, 0x88')
print('Match:', h[:8] == bytes([0x41, 0x4B, 0x3F, 0x4C, 0xEB, 0x5B, 0x5B, 0x88]))
"
```