# Testnet Deployment

Details of deployment on Neutron testnet `pion-1`.

## Astroport Liquidity Helper

- Checksum: `00e80be98cacf5e289ee5019a0d4da65547dd92f0627bb5624e9243c8a2241ef`
- Used optimized wasm downloaded from [Github Releases v0.2.0](https://github.com/apollodao/liquidity-helpers/releases/tag/v0.2.0).
- CodeID: `1455`
- Contract Address: `neutron1hd0a5z0dzndj7kwhf8qnxcgnjz97f2kyxad9y6vsmjxcd9egm7pq2tcyyz`

## CW DEX Router Astroport

- Checksum: `4256b997d7398f41ec035ff4c3924ff81fbac4cecdaa1898042eb157fb8d2284`
- Used optimized wasm built locally with rust-optimizer-arm 0.13.0.
- CodeID: `1456`
- Contract Address: `neutron1w7s7erc7th2pkjch78td7l5shjmec39x4tc0rs64ul7z2m23nyfs4s8pl8`

## Vault Zapper

- Checksum: `8233e7c689d4e5fe0d9850ae5511fab402e25ada90a0ff4971852e9b6d0b7c39`
- Used optimized wasm built on CI with rust-optimizer 0.13.0.
- CodeID: `1608`
- Contract Address: `neutron1g0g25nry5aqy9k0fc0auk0hsnzv09dl0n7c6n424vvyfyd567xhshghwlx`

## Locked Astroport Vault

### `0.1.0-rc.1`
- Checksum: `160d7604aaaa34bda8f986b04be6a7afb4e056acc982c676b44fa78165277e8b`
- CodeID: `1460`
- Used optimized wasm built locally with rust-optimizer-arm 0.13.0.

### `0.1.0-rc.2`
- Checksum: `d8be8a2311500aa1fc38d177c024eafc4d1f8185b752f5b654f2b367cfab6cc7`
- CodeID: `1481`
- Used optimized wasm built locally with rust-optimizer-arm 0.13.0.

### ASTRO/NTRN Unlocked Vault

- Contract Address: `neutron1675p0u4eflqgvfwf3tlk8rhkvfjggrenjlc4jqtyjgsrfp9kyrus6r8hly`
- CodeID: `1481`

### AXL/NTRN Unlocked Vault

- Contract Address: `neutron17dhpuf4fduc3mshw3e8t0wymkp42sz82uwmr8865r6hngm4fk4yscrmgyz`
- CodeID: `1481`

### wstETH/ETH 5min Locked Vault

- Contract Address: `neutron18repwf8rfsu6qsj6avyxe7r5n9h0jzqza85yzxmsn5uj59f42nes3u5nn3`
- CodeID: `1481`
