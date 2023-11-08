# Testnet Deployment

Details of deployment on Neutron testnet `pion-1`.

## Astroport Liquidity Helper

- Checksum: `8ac6ac64f14c57d643d5c5b5638c829ae1163c37390c65b16a67c5493a47cd98`
- Used optimized wasm downloaded from [Github Releases v0.3.1](https://github.com/apollodao/liquidity-helpers/releases/tag/v0.3.1).
- CodeID: `1943`
- Contract Address: `neutron1hd0a5z0dzndj7kwhf8qnxcgnjz97f2kyxad9y6vsmjxcd9egm7pq2tcyyz`

## CW DEX Router Astroport

- Checksum: `f69d9695d854f0f9a67640ca26a0bceec255a4252fca113915c80c1f9cc401d7`
- Used optimized wasm downloaded from [Github Releases v0.3.0](https://github.com/apollodao/cw-dex-router/releases/tag/v0.3.0).
- CodeID: `1944`
- Contract Address: `neutron1w7s7erc7th2pkjch78td7l5shjmec39x4tc0rs64ul7z2m23nyfs4s8pl8`

## Vault Zapper

- Checksum: `0185ac0713aa9420399cda317fd88d7cf0a3aa1fc02db3bfde71f1469158ead7`
- Used optimized wasm downloaded from [Github Releases v0.2.0](https://github.com/apollodao/vault-zapper/releases/tag/v0.2.0)
- CodeID: `1980`
- Contract Address: `neutron1uqyypp57zlel8ydwctypdy2k0eu7ru88ggpc5cjyrf92dr97zzaq2an2yd`

## Locked Astroport Vault

### `0.1.0-rc.1`
- Checksum: `160d7604aaaa34bda8f986b04be6a7afb4e056acc982c676b44fa78165277e8b`
- CodeID: `1460`
- Used optimized wasm built locally with rust-optimizer-arm 0.13.0.

### `0.1.0-rc.2`
- Checksum: `d8be8a2311500aa1fc38d177c024eafc4d1f8185b752f5b654f2b367cfab6cc7`
- CodeID: `1481`
- Used optimized wasm built locally with rust-optimizer-arm 0.13.0.

### `0.2.0`
- Checksum: `609cd0175fb2b5cefdc2bb24b8b3c91d783e88458edaa2d0d20f0232ecccfdf6`
- CodeID: `1945`
- Used optimized wasm built on CI with workspace-optimizer 0.13.0.

### ASTRO/NTRN Unlocked Vault

- Contract Address: `neutron1675p0u4eflqgvfwf3tlk8rhkvfjggrenjlc4jqtyjgsrfp9kyrus6r8hly`
- CodeID: `1481`

### AXL/NTRN Unlocked Vault

- Contract Address: `neutron1rylsg4js5nrm4acaqez5v95mv279lpfrstfupwqykkg6mcyt6lsqxafdcf`
- CodeID: `1945`

### wstETH/ETH 5min Locked Vault

- Contract Address: `neutron1zzwzqehc5nhyv6wztfr63etvfa8ujmt6h0m488ttdy2tmde5gdqsqe25yn`
- CodeID: `1945`

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
