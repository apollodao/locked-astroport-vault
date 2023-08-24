use cosmwasm_std::{
    Addr, BlockInfo, Deps, MessageInfo, Order, StdError, StdResult, Storage, Uint128,
};
use cw20::Expiration;
use cw_storage_plus::{Bound, Index, IndexList, IndexedMap, Item, MultiIndex};
use cw_vault_standard::extensions::lockup::UnlockingPosition;

// Settings for pagination
const DEFAULT_LIMIT: u32 = 10;

/// An unlocking position for a user that can be claimed once it has matured.
pub type Claim = UnlockingPosition;

/// A struct for handling the addition and removal of claims, as well as
/// querying and force unlocking of claims.
pub struct Claims<'a> {
    /// All currently unclaimed claims, both unlocking and matured. Once a claim
    /// is claimed by its owner after it has matured, it is removed from this
    /// map.
    claims: IndexedMap<'a, u64, Claim, ClaimIndexes<'a>>,
    // Counter of the number of claims. Used as a default value for the ID of a new
    // claim. This is monotonically increasing and is not decremented when a claim is removed
    // It represents the number of claims that have been created since creation of the `Claims`
    // instance.
    next_claim_id: Item<'a, u64>,
}

/// Helper struct for indexing claims. Needed by the [`IndexedMap`]
/// implementation.
pub struct ClaimIndexes<'a> {
    /// Index mapping an address to all claims for that address.
    pub owner: MultiIndex<'a, Addr, Claim, u64>,
}

impl<'a> IndexList<Claim> for ClaimIndexes<'a> {
    fn get_indexes(&'_ self) -> Box<dyn Iterator<Item = &'_ dyn Index<Claim>> + '_> {
        let v: Vec<&dyn Index<Claim>> = vec![&self.owner];
        Box::new(v.into_iter())
    }
}

impl<'a> Claims<'a> {
    /// Create a new Claims instance
    ///
    /// ## Arguments
    /// * `claims_namespace` - The key to use for the the primary key (u64
    ///   lockup ID)
    /// * `num_claims_key` - The key to use for the index value (owner addr)
    pub fn new(
        claims_namespace: &'a str,
        claims_index_namespace: &'a str,
        num_claims_key: &'a str,
    ) -> Self {
        let indexes = ClaimIndexes {
            owner: MultiIndex::new(
                |_pk, d| d.owner.clone(),
                claims_namespace,
                claims_index_namespace,
            ),
        };

        Self {
            claims: IndexedMap::new(claims_namespace, indexes),
            next_claim_id: Item::new(num_claims_key),
        }
    }

    /// Create a new claim and save it to the claims map.
    pub fn create_claim(
        &self,
        storage: &mut dyn Storage,
        addr: &Addr,
        amount: Uint128,
        release_at: Expiration,
    ) -> StdResult<Claim> {
        let id = self.next_claim_id.may_load(storage)?.unwrap_or_default();

        self.next_claim_id.save(storage, &(id + 1))?;

        let claim = Claim {
            owner: addr.clone(),
            id,
            release_at,
            base_token_amount: amount,
        };

        self.claims.save(storage, id, &claim)?;

        Ok(claim)
    }

    /// Redeem claim for the underlying tokens
    ///
    /// ## Arguments
    /// * `lock_id` - The id of the claim
    ///
    /// ## Returns
    /// Returns the amount of tokens redeemed if `info.sender` is the `owner` of
    /// the claim and the `release_at` time has passed, else returns an
    /// error. Also returns an error if a claim with the given `lock_id` does
    /// not exist.
    pub fn claim_tokens(
        &self,
        storage: &mut dyn Storage,
        block: &BlockInfo,
        info: &MessageInfo,
        id: u64,
    ) -> StdResult<Uint128> {
        let claim = self.claims.load(storage, id)?;

        // Ensure the claim is owned by the sender
        if claim.owner != info.sender {
            return Err(StdError::generic_err("Claim not owned by sender"));
        }

        // Check if the claim is expired
        if !claim.release_at.is_expired(block) {
            return Err(StdError::generic_err("Claim has not yet matured."));
        }

        // Remove the claim from the map
        self.claims.remove(storage, id)?;

        Ok(claim.base_token_amount)
    }

    /// Bypass expiration and claim `claim_amount`. Should only be called if the
    /// caller is whitelisted. Will return an error if the claim does not exist
    /// or if the caller is not the owner of the claim.
    /// TODO: Move whitelist logic into Claims struct? That way we won't need to
    /// have a separate ForceUnlock message.
    pub fn force_claim(
        &self,
        storage: &mut dyn Storage,
        info: &MessageInfo,
        lock_id: u64,
        claim_amount: Option<Uint128>,
    ) -> StdResult<Uint128> {
        let mut lockup = self.claims.load(storage, lock_id)?;

        // Ensure the claim is owned by the sender
        if lockup.owner != info.sender {
            return Err(StdError::generic_err("Claim not owned by sender"));
        }

        let claimable_amount = lockup.base_token_amount;

        let claimed = claim_amount.unwrap_or(claimable_amount);

        let left_after_claim = claimable_amount.checked_sub(claimed).map_err(|x| {
            StdError::generic_err(format!(
                "Claim amount is greater than the claimable amount: {}",
                x
            ))
        })?;

        if left_after_claim > Uint128::zero() {
            lockup.base_token_amount = left_after_claim;
            self.claims.save(storage, lock_id, &lockup)?;
        } else {
            self.claims.remove(storage, lock_id)?;
        }

        Ok(claimed)
    }

    // ========== Query functions ==========

    /// Query lockup by id
    pub fn query_claim_by_id(&self, deps: Deps, lockup_id: u64) -> StdResult<UnlockingPosition> {
        self.claims.load(deps.storage, lockup_id)
    }

    /// Reads all claims for an owner. The optional arguments `start_after` and
    /// `limit` can be used for pagination if there are too many claims to
    /// return in one query.
    ///
    /// # Arguments
    /// - `owner` - The owner of the claims
    /// - `start_after` - Optional id of the claim to start the query after
    /// - `limit` - Optional maximum number of claims to return
    pub fn query_claims_for_owner(
        &self,
        deps: Deps,
        owner: &Addr,
        start_after: Option<u64>,
        limit: Option<u32>,
    ) -> StdResult<Vec<(u64, Claim)>> {
        let limit = limit.unwrap_or(DEFAULT_LIMIT) as usize;
        let start: Option<Bound<u64>> = start_after.map(Bound::exclusive);

        self.claims
            .idx
            .owner
            .prefix(owner.clone())
            .range(deps.storage, start, None, Order::Ascending)
            .take(limit)
            .collect::<StdResult<Vec<_>>>()
    }
}

#[cfg(test)]
mod tests {
    use cosmwasm_std::testing::{
        mock_dependencies, mock_env, mock_info, MockApi, MockQuerier, MockStorage,
    };
    use cosmwasm_std::{Addr, OwnedDeps, Uint128};
    use cw_utils::Expiration;

    use test_case::test_case;

    use super::*;

    const OWNER: &str = "owner";
    const NOT_OWNER: &str = "not_owner";

    const CLAIMS: &str = "claims";
    const CLAIMS_INDEX: &str = "claims_index";
    const NUM_CLAIMS: &str = "num_claims";
    const BASE_TOKEN_AMOUNT: Uint128 = Uint128::new(100);
    const EXPIRATION: Expiration = Expiration::AtHeight(100);

    fn create_claim() -> (
        OwnedDeps<MockStorage, MockApi, MockQuerier>,
        Claims<'static>,
        Claim,
    ) {
        let mut deps = mock_dependencies();

        let claims = Claims::new(CLAIMS, CLAIMS_INDEX, NUM_CLAIMS);

        let claim = claims
            .create_claim(
                &mut deps.storage,
                &Addr::unchecked(OWNER),
                BASE_TOKEN_AMOUNT,
                EXPIRATION,
            )
            .unwrap();

        (deps, claims, claim)
    }

    #[test]
    fn test_create_claim() {
        let (deps, claims, claim) = create_claim();

        // Check that the claim was created
        let stored_claim = claims.claims.load(&deps.storage, claim.id).unwrap();

        // Assert that the claim was saved to storage
        assert_eq!(stored_claim, claim);

        // Assert that the claim has the correct values
        assert_eq!(
            stored_claim,
            Claim {
                id: 0,
                owner: Addr::unchecked(OWNER),
                base_token_amount: BASE_TOKEN_AMOUNT,
                release_at: EXPIRATION,
            }
        );
    }

    #[test_case(100, NOT_OWNER => Err(StdError::generic_err("Claim not owned by sender")); "claim not owned by sender")]
    #[test_case(100, OWNER => Ok(BASE_TOKEN_AMOUNT) ; "claim owned by sender")]
    #[test_case(99, OWNER => Err(StdError::generic_err("Claim has not yet matured.")); "claim not yet matured")]
    fn test_claim_tokens(block_height: u64, sender: &str) -> StdResult<Uint128> {
        let mut env = mock_env();
        env.block.height = block_height;
        let info = mock_info(sender, &[]);

        let (mut deps, claims, _claim) = create_claim();

        match claims.claim_tokens(&mut deps.storage, &env.block, &info, 0) {
            Ok(amount) => {
                // Assert that the claim was deleted
                assert!(claims.claims.load(&deps.storage, 0).is_err());
                Ok(amount)
            }
            Err(err) => {
                // Assert that the claim was not deleted
                assert!(claims.claims.load(&deps.storage, 0).is_ok());
                Err(err)
            }
        }
    }

    #[test_case(None, OWNER => Ok(BASE_TOKEN_AMOUNT); "sender is owner")]
    #[test_case(None, NOT_OWNER => Err(StdError::generic_err("Claim not owned by sender")); "sender is not owner")]
    #[test_case(Some(Uint128::new(99u128)), OWNER => Ok(Uint128::new(99u128)); "sender is owner and amount is less than base token amount")]
    fn test_force_unlock(claim_amount: Option<Uint128>, sender: &str) -> StdResult<Uint128> {
        let info = mock_info(sender, &[]);

        let (mut deps, claims, _claim) = create_claim();

        match claims.force_claim(&mut deps.storage, &info, 0, claim_amount) {
            Ok(amount) => {
                // Assert that the claim was deleted if entire amount was unlocked
                if amount == BASE_TOKEN_AMOUNT {
                    assert!(claims.claims.load(&deps.storage, 0).is_err());
                } else {
                    assert_eq!(
                        claims
                            .claims
                            .load(&deps.storage, 0)
                            .unwrap()
                            .base_token_amount,
                        BASE_TOKEN_AMOUNT - amount
                    );
                }
                Ok(amount)
            }
            Err(err) => {
                // Assert that the claim was not deleted
                assert!(claims.claims.load(&deps.storage, 0).is_ok());
                Err(err)
            }
        }
    }

    #[test_case(0 => Ok(Claim {id: 0, owner: Addr::unchecked(OWNER), base_token_amount: BASE_TOKEN_AMOUNT, release_at: EXPIRATION}); "claim exists")]
    #[test_case(1 => matches Err(_); "claim does not exist")]
    fn test_query_claim_by_id(id: u64) -> StdResult<Claim> {
        let (deps, claims, _claim) = create_claim();

        // Query the claim
        claims.query_claim_by_id(deps.as_ref(), id)
    }

    fn claims(start_id: u64, n: u32) -> Vec<Claim> {
        let mut claims = Vec::new();
        for i in start_id..(start_id + n as u64) {
            claims.push(Claim {
                id: i,
                owner: Addr::unchecked(OWNER),
                base_token_amount: BASE_TOKEN_AMOUNT,
                release_at: EXPIRATION,
            });
        }
        claims
    }

    #[test_case(OWNER, None, None => Ok(claims(0, DEFAULT_LIMIT)); "default pagination")]
    #[test_case(OWNER, None, Some(31) => Ok(claims(0, 31)); "pagination with limit")]
    #[test_case(OWNER, Some(1), None => Ok(claims(2, DEFAULT_LIMIT)); "pagination with start id")]
    #[test_case(OWNER, Some(1), Some(31) => Ok(claims(2, 31)); "pagination with start id and limit")]
    fn test_query_claims_for_owner(
        owner: &str,
        start_after: Option<u64>,
        limit: Option<u32>,
    ) -> StdResult<Vec<Claim>> {
        let mut deps = mock_dependencies();

        // Create 100 claims for owner
        let claims = Claims::new(CLAIMS, CLAIMS_INDEX, NUM_CLAIMS);
        let owner = Addr::unchecked(owner);
        for _ in 0..100 {
            claims
                .create_claim(&mut deps.storage, &owner, BASE_TOKEN_AMOUNT, EXPIRATION)
                .unwrap();
        }

        // Query the claims without using pagination arguments
        claims
            .query_claims_for_owner(deps.as_ref(), &owner, start_after, limit)
            .map(|claims| claims.iter().map(|c| c.1.clone()).collect())
    }
}
