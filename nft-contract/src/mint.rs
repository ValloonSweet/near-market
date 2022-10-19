use near_sdk::near_bindgen;

use crate::*;

#[near_bindgen]
impl Contract {
    #[payable]
    pub fn nft_mint(
        &mut self,
        token_id: TokenId,
        metadata: TokenMetadata,
        receiver_id: AccountId,
        perpetual_royalties: Option<HashMap<AccountId, u32>>,
    ) {
        // measure the initial storage being used on the contract
        let initial_storage_usage = env::storage_usage();

        // create a royalty map to store in the tokne
        let mut royalty = HashMap::new();

        // if perpetual royalties were passed into the function
        if let Some(perpetual_royalties) = perpetual_royalties {
            assert!(perpetual_royalties.len() < 7, "Cannot add more than 6 perpetual royalty amounts");

            for (account, amount) in perpetual_royalties {
                royalty.insert(account, amount);
            }
        }

        // specify the token contract that contains the owner ID
        let token = Token {
            // set the owner ID equal to the receiver ID passed into the function
            owner_id: receiver_id,
            approved_account_ids: Default::default(),
            next_approval_id: 0,
            royalty
        };

        // insert the token ID and token struct and make sure that the token doesn't exist
        assert!(
            self.tokens_by_id.insert(&token_id, &token).is_none(),
            "Token already exists"
        );

        // insert the token ID and metadata
        self.token_metadata_by_id.insert(&token_id, &metadata);

        // call the internal method for adding the token to the owner
        self.internal_add_token_to_owner(&token.owner_id, &token_id);

        // calculate the required storage which was the used - initial
        let required_storage_in_bytes = env::storage_usage() - initial_storage_usage;

        // refund any excess storage if the user attached too much. Panic if they didn't attach enough to cover the required
        refund_deposit(required_storage_in_bytes);
    }
}
