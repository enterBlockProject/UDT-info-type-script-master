# UDT info type script

Script verifying UDT info cell on-chain as read-only with capsule (https://github.com/nervosnetwork/capsule).
* Only owner of UDT can make the info cell.
* There can be multiple info cells in different transactions with same UDT. 
  In this situation, Dapps can choose whether the UDT has some problem or not. 
  If dapps choose to read the info (assuming as the UDT has no problem), dapp can read first (oldest) info cell as correct. 

## structure
- args
  - type script hash of UDT
  
- data field (not little endian)
  - 8 bytes of Symbol
  - 1 byte of Decimal
  - not limited length of Name, any other information

## Script verify
1. Check if there is any group input cell. If exists, fail.
2. Check if there is only one group output cell. If not, fail.
3. Find UDT cell from input matching UDT info cell's args. If nothing matches, fail.
4. Check if the UDT is in owner mode from found UDT cell. If not owner mode, fail.
5. Check if there are at least 10 bytes in info cell's data field. If not, fail.


## Build contracts

``` sh
capsule build --release
```

## Run tests

``` sh
capsule test
```
