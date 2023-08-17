# Apollo Astroport Vault

This contract is a vault contract that implements the [cw-vault-standard](https://github.com/apollodao/cw-vault-standard). It is an autocompounding vault that takes in Astroport LP tokens as the base token. It claims any rewards from the Astroport generator as well as tokens that are sent directly to the contract (as long as they are present in the contracts config field `reward_tokens`) and sells these for more of the tokens that comprise the base LP token. It then provides liquidity with these to get more LP tokens and stakes them in the Astroport generator.

## License

This vault is licensed under the MariaDB Business Source License 1.1. See the [LICENSE](LICENSE) file for details.
