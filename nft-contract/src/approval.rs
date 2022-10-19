use crate::*;
use near_sdk::{ext_contract};

pub trait NonFungibleTokenCore {

    // approve an account ID to transfer a token on your behalf
    fn nft_approve(&mut self, token_id: TokenId, account_id: AccountId, msg: Option<String>);

    // check if the passed in account has access to approve the token ID
    fn nft_is_approved(&self, token_id: TokenId, approved_account_id: AccountId, approval_id: Option<u64>) -> bool;

    // revoke a specific account from transferring the token on your behalf
    fn nft_revoke(&mut self, token_id: TokenId, account_id: AccountId);

    // revoke all accounts from tranferring the token on your behalf
    fn nft_revoke_all(&mut self, token_id: TokenId);
}

#[ext_contract(ext_non_fungible_approval_receiver)]
trait NonFungibleTokenApprovalReceiver {
    // cross contract call to an external contract that is initiated during nft_approve
    fn nft_on_approve(
        &mut self,
        token_id: TokenId,
        owner_id: AccountId,
        approval_id: u64,
        msg: String
    );
}

#[near_bindgen]
impl NonFungibleTokenCore for Contract {
    // allow a specific account ID to approve a token on your behalf
    #[payable]
    fn nft_approve(&mut self, token_id: TokenId, account_id: AccountId, msg: Option<String>) {
        /*
            assert at least one yocto for security reasons - this will cause a redirect to the NEAR wallet.
            The User needs to attach enough to pay for storage on the contract
        */
        assert_at_least_one_yocto();

        // get the token object from the token ID
        let mut token = self.tokens_by_id.get(&token_id).expect("Not token");

        // make sure that the person calling the function is the owner of the token
        assert_eq!(
            &env::predecessor_account_id(),
            &token.owner_id,
            "Precedecessor must be the token owner."
        );

        // get the next approval ID if we need a new approval
        let approval_id: u64 = token.next_approval_id;

        // check if the account has been approved already for this token
        let is_new_approval = token
            .approved_account_ids
            .insert(account_id.clone(), approval_id)
            .is_none();

        // if it was a new approval, we need to calculate how much storage is being used to add the account
        let storage_used = if is_new_approval {
            bytes_for_approved_account_id(&account_id)
        } else {
            0
        };

        // increment the token's next approval ID by 1
        token.next_approval_id += 1;

        // insert the token back into the tokens_by_id collection
        self.tokens_by_id.insert(&token_id, &token);

        // refund any excess storage attached by the user.
        refund_deposit(storage_used);

        // if some message was passed into the function, we initiate a cross contract call
        if let Some(msg) = msg {
            ext_non_fungible_approval_receiver::ext(account_id)
                .nft_on_approve(
                    token_id,
                    token.owner_id,
                    approval_id,
                    msg
                ).as_return();
        }

    }

    fn nft_is_approved(
        &self,
        token_id: TokenId,
        approved_account_id: AccountId,
        approval_id: Option<u64>
    ) -> bool {
        // get token
        let token = self.tokens_by_id.get(&token_id).expect("No token");

        // get the approval number for the passed in account ID
        let approval = token.approved_account_ids.get(&approved_account_id);

        // if there was some approval ID found for the account ID
        if let Some(approval) = approval {
            // check id
            if let Some(approval_id) = approval_id {
                approval_id == *approval
            } else {
                true
            }
        } else {
            false
        }
    }

    #[payable]
    fn nft_revoke(&mut self, token_id: TokenId, account_id: AccountId) {
        // assert that the user attach exactly 1 yoctoNEAR for security reasons
        assert_one_yocto();

        // get token
        let mut token = self.tokens_by_id.get(&token_id).expect("No token");

        // get the caller
        let predecessor_account_id = env::predecessor_account_id();
        assert_eq!(
            &predecessor_account_id, &token.owner_id
        );

        // if the account ID was in the token's approval
        if token.approved_account_ids
            .remove(&account_id)
            .is_some() {
            refund_approved_account_ids_iter(predecessor_account_id, [account_id].iter());
            self.tokens_by_id.insert(&token_id, &token);
        }
    }

    #[payable]
    fn nft_revoke_all(&mut self, token_id: TokenId) {
        assert_one_yocto();

        let mut token = self.tokens_by_id.get(&token_id).expect("No token");

        let precedessor_account_id = env::predecessor_account_id();
        assert_eq!(&precedessor_account_id, &token.owner_id);

        if !token.approved_account_ids.is_empty() {
            refund_approved_account_ids(precedessor_account_id, &token.approved_account_ids);

            token.approved_account_ids.clear();

            self.tokens_by_id.insert(&token_id, &token);
        }
    }

}

