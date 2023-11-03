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
- CodeID: `1622`
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

# Mainet Deployment

Details of deployment on Neutron mainnet `neutron-1`.

## Astroport Liquidity Helper

- Version: `0.2.1`
- Checksum: `9403b46dec7facb7cbb9bea4f54a9644fa7e5e8db49c522ce326698324284820`
- Used optimized wasm downloaded from [Github Releases v0.2.1](https://github.com/apollodao/liquidity-helpers/releases/tag/v0.2.1).
- CodeID: `263`
- Contract Address: `` // TODO

## CW DEX Router Astroport

- Version: `0.2.0`
- Checksum: `87a9ee5692e0b0673dcdd5f499b50550cf2651a0e5e8c30b8d9edbfc3afee7dc`
- Used optimized wasm downloaded from [Github Releases v0.2.0](https://github.com/apollodao/cw-dex-router/releases/tag/v0.2.0).
- CodeID: `262`
- Contract Address: `` // TODO

## Vault Zapper

- Version: `0.1.0`
- Checksum: `c8206f95ee4bc87c93a92823e643c3ad9341beac4f714f0e331d0a14c0fdd418`
- Used wasm: Used optimized wasm downloaded from [Github Releases v0.1.0](https://github.com/apollodao/vault-zapper/releases/tag/v0.1.0).
- CodeID: `277`
- Contract Address: ``

## Locked Astroport Vault

### `0.1.0`

- Version: `0.1.0`
- Checksum: `8c40bc3e73b93d43423e3ebe4fe69b34b1bf46129acebc0f40a48bd9d53edc6e`
- CodeID: `266`
- Used optimized wasm built on CI with rust-optimizer 0.13.0.

### `0.1.1-rc.1`

- Version: `0.1.1-rc.1`
- Checksum: `6a2c35c6b7ab170c22c2a804a2b698ef9e2edeaf9f2d0d75c6917f42621caf8e`
- CodeID: `278`
- Used optimized wasm built locally with cosmwasm/rust-optimizer-arm64:0.13.0.

### AXL/NTRN Unlocked Vault

- Contract Address: `neutron12pdx3z009fx92kcsr8fhvnkh92w98n5f5xaecz3sn0kzxthz20js0llxjw`
- CodeID: `266`

### wstETH/axlWETH Unlocked Vault

- Contract Address: `neutron1jyk9sulr5wfyy0zp95cujupvennnc9xap79wkp6pwp7k2qmsmz2qw0wkrg`
- CodeID: `278`
