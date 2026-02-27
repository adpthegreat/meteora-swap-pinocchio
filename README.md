# Meteora Swaps 

Demonstrates a solana program to execute two swap in sequence atomically in a transaction using pinocchio, we use meteoras DLMM program as the swap venue, it was inspired by this reply to the tweet https://x.com/deanmlittle/status/2025787879461634315?s=20 to actually compare the normal double CPI method with a method where we introspect the former top level swap instruction


```
Pattern A: Both Swaps
[Router]
  ├─> Swap A (CPI)
  │     └─> SwapProgram
  └─> Swap B (CPI)
        └─> SwapProgram

Pattern B: Introspect + 1 CPI
[Swap A top-level]   
  └─> SwapProgram x2
[Router]
  └─> Swap B (CPI)
        └─> SwapProgram
```

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
`surfpool start` in this directory to start the local surfpool validator, to install surfpool visit 
[surfpool](surfpool.run)

Then to run the tests we run

```
yarn run test:1
yarn run test:2
```

## Note
- Fixed base64ct `cargo build-sbf` build error by downgrading it https://solana.stackexchange.com/questions/23578/how-to-build-with-solana-program-3-0-0-cargo-build-sbf-3-0-6-fails-with-edition


## Quick script to convert base58 address to hex
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
```python
python3 -c "
import hashlib
h = hashlib.sha256(b'global:swap2').digest()
print('Expected: ', ', '.join(f'0x{b:02X}' for b in h[:8]))
print('Got:      ', '0x41, 0x4B, 0x3F, 0x4C, 0xEB, 0x5B, 0x5B, 0x88')
print('Match:', h[:8] == bytes([0x41, 0x4B, 0x3F, 0x4C, 0xEB, 0x5B, 0x5B, 0x88]))
"
```

