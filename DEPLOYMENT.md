# Testnet Deployment

Details of deployment on Neutron testnet `pion-1`.

## Astroport Liquidity Helper

- Version: `0.3.1`
- Checksum: `8ac6ac64f14c57d643d5c5b5638c829ae1163c37390c65b16a67c5493a47cd98`
- Used optimized wasm downloaded from [Github Releases v0.3.1](https://github.com/apollodao/liquidity-helpers/releases/tag/v0.3.1).
- CodeID: `1943`
- Contract Address: `neutron1hd0a5z0dzndj7kwhf8qnxcgnjz97f2kyxad9y6vsmjxcd9egm7pq2tcyyz`

## CW DEX Router Astroport

- Version: `0.3.0`
- Checksum: `f69d9695d854f0f9a67640ca26a0bceec255a4252fca113915c80c1f9cc401d7`
- Used optimized wasm downloaded from [Github Releases v0.3.0](https://github.com/apollodao/cw-dex-router/releases/tag/v0.3.0).
- CodeID: `1944`
- Contract Address: `neutron1w7s7erc7th2pkjch78td7l5shjmec39x4tc0rs64ul7z2m23nyfs4s8pl8`

## Vault Zapper

- Version: `0.2.0`
- Checksum: `0185ac0713aa9420399cda317fd88d7cf0a3aa1fc02db3bfde71f1469158ead7`
- Used optimized wasm downloaded from [Github Releases v0.2.0](https://github.com/apollodao/vault-zapper/releases/tag/v0.2.0).
- CodeID: `1980`
- Contract Address: `neutron1uqyypp57zlel8ydwctypdy2k0eu7ru88ggpc5cjyrf92dr97zzaq2an2yd`

## Reward Distributor

### `0.2.0`
- Checksum: `0977f0cacf5e47fb9b0d4c7a4c688864240b7d7cf9b727d3a7454ef83bcc552c`
- Used optimized wasm downloaded from [Github Releases v0.2.0](https://github.com/apollodao/neutron-reward-distributor/releases/tag/v0.2.0).
- CodeID: `2003`

### Reward Distributor for wstETH/ETH 5min Locked Vault
- CodeID: `2003`
- Contract Address: `neutron16x9l074xt9chkd8f2t83ehupj73j4spkdflsh3eqzx5fmmjcm7jqu37u6a`
- Distribution target: `neutron1zzwzqehc5nhyv6wztfr63etvfa8ujmt6h0m488ttdy2tmde5gdqsqe25yn` (wstETH/ETH 5min Locked Vault)
- Reward Vault: `neutron1rylsg4js5nrm4acaqez5v95mv279lpfrstfupwqykkg6mcyt6lsqxafdcf` (AXL/NTRN Unlocked Vault)

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

- Version: `0.3.1`
- Checksum: `8ac6ac64f14c57d643d5c5b5638c829ae1163c37390c65b16a67c5493a47cd98`
- Used optimized wasm downloaded from [Github Releases v0.3.1](https://github.com/apollodao/liquidity-helpers/releases/tag/v0.3.1).
- CodeID: `461`
- Contract Address: `neutron12dqtnwn8ys5x8zdf65h0neumusfnkzkh79x6wuxyg94kjqej7a7qrft8a6`

## CW DEX Router Astroport

- Version: `0.3.0`
- Checksum: `f69d9695d854f0f9a67640ca26a0bceec255a4252fca113915c80c1f9cc401d7`
- Used optimized wasm downloaded from [Github Releases v0.3.0](https://github.com/apollodao/cw-dex-router/releases/tag/v0.3.0).
- CodeID: `460`
- Contract Address: `neutron1myakw8cpyr4430ncyg33mqzpjuzeqhhsp5auv8snc9ekwlr8vtuq9zgve0`

## Vault Zapper

- Version: `0.2.0`
- Checksum: `0185ac0713aa9420399cda317fd88d7cf0a3aa1fc02db3bfde71f1469158ead7`
- Used wasm: Used optimized wasm downloaded from [Github Releases v0.2.0](https://github.com/apollodao/vault-zapper/releases/tag/v0.2.0).
- CodeID: `462`
- Contract Address: `neutron1t0ua4m4l8d8d39yjvvulzd73cp5lzdlx4s79vk7d70m9zsfjvpaq3n6eyr`

## Reward Distributor

- Version: `0.2.0`
- Checksum: `0977f0cacf5e47fb9b0d4c7a4c688864240b7d7cf9b727d3a7454ef83bcc552c`
- Used optimized wasm downloaded from [Github Releases v0.2.0](https://github.com/apollodao/neutron-reward-distributor/releases/tag/v0.2.0).
- CodeID: `499`

### Reward Distributor for wstETH/axlWETH 7d Locked Vault

- CodeID: `499`
- Contract Address: `neutron1hztlr9f5gesalqqzzdd0klr2pmvrzrxpv9ae53rzv22ycgrzng4stag887`

### Reward Distributor for NTRN/wstETH 7d Locked Vault

- CodeID: `499`
- Contract Address: `neutron1qv9t4s9hzv3vphe6f5grffv5n0uag49qjfw9n4r66v769cq48fwqhtdhsq`

### stTIA Reward Distributor for stTIA/TIA Capped Vault

- CodeID: `499`
- Contract Address: `neutron1vhgyz0ttwcxpt8q9kyr4uy5fktmq284jjfma90q87a49zrcn7nfs98w8eg`

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

### `0.2.0`
- Version: `0.2.0`
- Checksum: `609cd0175fb2b5cefdc2bb24b8b3c91d783e88458edaa2d0d20f0232ecccfdf6`
- CodeID: `451`
- Used optimized wasm built on CI with workspace-optimizer 0.13.0.

### `0.4.0`
- Version: `0.4.0`
- Checksum: `d2f3e6ac9a1af422aa5be3701dbe417482720d862c2422684fd4ec8ce6929477`
- CodeID: `709`
- Used optimized wasm built on CI with workspace-optimizer 0.13.0.

### `0.4.1
- Version: `0.4.1`
- Checksum: `ce5a270bae7ecf44c76a3674c8838f1687cdc09b2b7310e81810b90358eb55e6`
- CodeID: `714`
- Used optimized wasm built on CI with workspace-optimizer 0.13.0.

### AXL/NTRN Unlocked Vault

- Contract Address: `neutron12pdx3z009fx92kcsr8fhvnkh92w98n5f5xaecz3sn0kzxthz20js0llxjw`
- CodeID: `266`

### XYK wstETH/axlWETH Unlocked Vault

- Contract Address: `neutron1jyk9sulr5wfyy0zp95cujupvennnc9xap79wkp6pwp7k2qmsmz2qw0wkrg`
- CodeID: `278`

### XYK ASTRO/axlUSDC Unlocked Vault

- Contract Address: `neutron135nkp0fth0vtertv7ngvkkgc4cwamp2tpnmjdlppat0047f9wjmqxeu9p8`
- CodeID: `451`

### PCL wstETH/axlWETH 7d Vault

- Contract Address: `neutron1yvhe4f0q3swtf37pkf9kku59l52nevr3trxs62vah004a08pkl8qlaccc7`
- CodeID: `714`
- Contract admin: `neutron1qnpwxhrgd8mmsfgql7df6kusgjr3wvm4trl05xu260seelwh845qtqqq9t`

### PCL NTRN/wstETH 7d Vault

- Contract Address: `neutron17vedy2clhctw0654k93m375ud7h5jsy8nj9gnlkjnyd4mcfnfrdql226al`
- CodeID: `714`
- Contract admin: `neutron1qnpwxhrgd8mmsfgql7df6kusgjr3wvm4trl05xu260seelwh845qtqqq9t`

### PCL stTIA/TIA 0d Capped Vault

- Contract Address: `neutron1qzf6t478xuutq0ahkm07pl2y2tctreccrlafkrl38k4cafk3rgdq3lfky5`
- CodeID: `709`
- Contract admin: `neutron1qnpwxhrgd8mmsfgql7df6kusgjr3wvm4trl05xu260seelwh845qtqqq9t`

### XYK (stTIA/TIA)VT/APOLLO 0d Vault

- Contract Address: `neutron19h6eltj6dem7a6jp6r2plwl95fgcryxeylvnm8ezlglezxzzkzrsnkj006`
- CodeID: `714`
- Contract admin: `neutron1qnpwxhrgd8mmsfgql7df6kusgjr3wvm4trl05xu260seelwh845qtqqq9t`
