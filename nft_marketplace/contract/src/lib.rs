use std::convert::TryFrom;
use near_sdk::borsh::{self, BorshSerialize, BorshDeserialize};
use near_sdk::collections::{LookupMap};
use near_sdk::json_types::{U128, ValidAccountId};
use near_sdk::serde::{Serialize, Deserialize};
use near_sdk::{env, ext_contract, near_bindgen, AccountId, Gas, Balance, PanicOnDefault, Promise, PromiseResult};

const RESOLVE_GAS_TRANSFER: Gas = 10_000_000_000_000;
const NFT_TRANSFER_GAS: Gas = 25_000_000_000_000 + RESOLVE_GAS_TRANSFER;
const ZERO_DEPOSIT: Balance = 0;
const MIN_ATTACHED_AMOUNT: u128 = 100_000_000_000_000_000_000_000; // 0.1 Near

pub type NftId = String;

#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct Sale {
    pub owner: AccountId, 
    pub beneficiary: AccountId,
    pub price: Balance,
    pub deposit: Balance,
}

#[near_bindgen]
#[derive(BorshSerialize, BorshDeserialize, PanicOnDefault)]
pub struct Contract {
    pub owner: AccountId,
    pub sales: LookupMap<NftId, Sale>
}

#[near_bindgen]
impl Contract {
    #[init]
    pub fn new(owner: ValidAccountId) -> Self {
        assert!(!env::state_exists(), "Already initialized");
        Self {
            owner: owner.into(),
            sales: LookupMap::new(b"s".to_vec()),
        }
    }

    #[payable]
    pub fn add_sale(
        &mut self,
        token_contract_id: ValidAccountId,
        token_id: String,
        price: U128,
        on_behalf_of: Option<AccountId>) {

        let deposit = env::attached_deposit();
        assert!(deposit >= MIN_ATTACHED_AMOUNT, "Attach at least 0.1 NEAR as deposit to listt sale");
        let contract_id: AccountId = token_contract_id.into();

        let mut owner = env::predecessor_account_id();
        if let Some(on_behalf_of) = on_behalf_of {
            owner = on_behalf_of;
        }

        let item_sale = Sale {
            owner,
            beneficiary: env::predecessor_account_id(),
            price: price.into(),
            deposit
        };

        self.sales.insert(&format!("{}:{}", contract_id, token_id), &item_sale);
    }

    pub fn remove_sale(
        &mut self,
        token_contract_id: ValidAccountId,
        token_id: String) {
        let contract_id: AccountId = token_contract_id.into();
        let sale = self.sales.remove(&format!("{}:{}", contract_id, token_id)).expect("No sale");

        assert_eq!(
            env::predecessor_account_id(),
            sale.owner,
            "Must be the owner of this NFT"
        );

        Promise::new(sale.owner).transfer(sale.deposit);
    }

    #[payable]
    pub fn purchase(
        &mut self,
        token_contract_id: ValidAccountId,
        token_id: String
        ) -> Promise {
        let contract_id: AccountId = token_contract_id.into();
        let sale = self.sales.get(
            &format!("{}:{}", contract_id, token_id.clone())
        ).expect("No sale");

        let deposit = env::attached_deposit();
        assert_eq!(
            env::attached_deposit(),
            sale.price,
            "Must pay exactly the sale amount {}", deposit
        );

        let predecessor = env::predecessor_account_id();
        let receiver_id = ValidAccountId::try_from(predecessor.clone()).unwrap();
        let owner = ValidAccountId::try_from(sale.owner).unwrap();
        let memo: String = "Sold by Market".to_string();

        ext_transfer::nft_transfer(
            receiver_id,
            token_id.clone(),
            owner,
            memo,
            &contract_id,
            1,
            env::prepaid_gas() - NFT_TRANSFER_GAS
        ).then(ext_self::nft_resolve_purchase(
            contract_id,
            token_id,
            predecessor,
            &env::current_account_id(),
            ZERO_DEPOSIT,
            RESOLVE_GAS_TRANSFER
        ))
    }

    pub fn nft_resolve_purchase(
        &mut self,
        token_contract_id: ValidAccountId,
        token_id: NftId,
        buyer_id: AccountId
        ) -> bool {
        env::log(format!("Promise Result {:?}", env::promise_result(0)).as_bytes());

        if let PromiseResult::Successful(_value) = env::promise_result(0) {
            let sale = self.sales.remove(
                &format!("{}:{}", token_contract_id, token_id)).expect("No sale");
            Promise::new(sale.beneficiary).transfer(sale.price + sale.deposit);
            return true;
        }

        let sale = self.sales.get(
            &format!("{}:{}", token_contract_id, token_id)).expect("No sale");
        Promise::new(buyer_id).transfer(sale.price);
        return false;
    }

    pub fn get_sale(
        &self,
        token_contract_id: ValidAccountId,
        token_id: String
        ) -> Sale {
        let contract_id: AccountId = token_contract_id.into();
        self.sales.get(
            &format!("{}:{}", contract_id, token_id.clone())).expect("No sale")
    }
}

#[ext_contract(ext_self)]
trait ResolvePurchase {
    fn nft_resolve_purchase(
        &mut self,
        token_contract_id: AccountId,
        token_id: NftId,
        buyer_id: AccountId
    ) -> Promise;
}

#[ext_contract(ext_transfer)]
trait ExtTransfer {
    fn nft_transfer(
        &mut self,
        receiver_id: ValidAccountId,
        token_id: NftId,
        enforce_owner_id: ValidAccountId,
        memo: String
        );
}
