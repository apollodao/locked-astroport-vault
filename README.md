# Apollo Astroport Vault

This contract is a vault contract that implements the [cw-vault-standard](https://github.com/apollodao/cw-vault-standard). It is an autocompounding vault that takes in Astroport LP tokens as the base token. It claims any rewards from the Astroport generator as well as tokens that are sent directly to the contract (as long as they are present in the contracts config field `reward_tokens`) and sells these for more of the tokens that comprise the base LP token. It then provides liquidity with these to get more LP tokens and stakes them in the Astroport generator.

This vault has an optional configurable `lockup_duration`. If this duration is set to some value higher than zero, the deposited funds will be locked this amount of time from when the user redeems their vault tokens.

## License

This vault is licensed under the MariaDB Business Source License 1.1. See the [LICENSE](LICENSE) file for details.
