# Contract Architecture

## Source files

|File  |Description |
|---|---|
| approval.rs | Has the functions that controls the access and transfers of non-fungible tokens  |
|enumeration.rs   |Contains the methods to list NFT tokens and their owners   |
|lib.rs   |Holds the smart contract initialization functions   |
|metadata.rs   |Defines the token and metadata structure   |
|mint.rs|contains token minting logic.|
|nft_core.rs|Core logic that allows you to transfer NFTs between users|
|royalty.rs|Contains payout-related functions|
| | |


# Minting


## Minting Tasks

To mint a non-fungible token, in the most simple way possible, a contract needs to be able to associate a token with an owner on the blokchain. Thi means you'll need:
- A way to keep track of tokens and other information on the contract
- A way to store information for each token such as `metadata` (more on later)
- A way to link a token with an owner

## Storing information on the contract

nft-contract/src/lib.rs
```
#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
pub struct Contract {
    //contract owner
    pub owner_id: AccountId,

    //keeps track of all the token IDs for a given account
    pub tokens_per_owner: LookupMap<AccountId, UnorderedSet<TokenId>>,

    //keeps track of the token struct for a given token ID
    pub tokens_by_id: LookupMap<TokenId, Token>,

    //keeps track of the token metadata for a given token ID
    pub token_metadata_by_id: UnorderedMap<TokenId, TokenMetadata>,

    //keeps track of the metadata for the contract
    pub metadata: LazyOption<NFTContractMetadata>,
}
```

This allows you to get the information stored in these data structures from anywhere in the contract.
- tokens_per_owner: allows you to keep track of tokens owned by any account
- tokens_by_id: returns all the information about a specific token
- token_metadata_by_id: returns just the metadata for a specific token

