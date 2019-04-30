#![cfg_attr(not(any(test, feature = "std")), no_std)]

use ink_core::{
    env::{self, AccountId, Balance},
    memory::format,
    storage,
};
use ink_lang::contract;
use parity_codec::{Decode, Encode};

/// Events deposited by the NFToken contract
#[derive(Encode, Decode)]
enum Event {
    /// Emits when the owner of the contract mints tokens
    Mint { owner: AccountId, value: Balance },
    /// Emits when a transfer has been made.
    Transfer {
        from: Option<AccountId>,
        to: Option<AccountId>,
        token_id: u64,
    },
}

/// Deposits an NFToken event.
fn deposit_event(event: Event) {
    env::deposit_raw_event(&event.encode()[..])
}

contract! {
    /// Storage values of the contract
    struct NFToken {
        /// Total tokens minted
        owner: storage::Value<AccountId>,
        /// Mapping: token_id(u64) -> owner (AccountID)
        id_to_owner: storage::HashMap<Balance, AccountId>,
        /// Mapping: owner(AccountID) -> tokenCount (Balance)
        owner_to_token_count: storage::HashMap<AccountId, Balance>,
        /// Total tokens minted
        total_minted: storage::Value<Balance>,
    }

    /// compulsary Demploy method
    impl Deploy for NFToken {
        /// Initializes our initial total minted value to 0.
        fn deploy(&mut self, init_value: Balance) {
            self.total_minted.set(0);
            // set ownership of contract
            self.owner.set(env::caller());
            // mint initial tokens
            if init_value > 0 {
              self.mint_impl(env.caller(), init_value);
            }
        }
    }

    /// Public methods
    impl NFToken {

        /// Return the total amount of tokens ever minted
        pub(external) fn total_minted(&self) -> Balance {
            let total_minted = *self.total_minted;
            env.println(&format!("NFToken::total_minted = {:?}", total_minted));
            total_minted
        }

        /// Return the balance of the given address.
        pub(external) fn balance_of(&self, owner: AccountId) -> Balance {
            let balance = *self.owner_to_token_count.get(&owner).unwrap_or(&0);
            env.println(&format!("NFToken::balance_of(owner = {:?}) = {:?}", owner, balance));
            balance
        }

        /// Transfers a token_id to a specified address from the caller
        pub(external) fn transfer(&mut self, to: AccountId, token_id: u64) -> bool {
            env.println(&format!(
                "NFToken::transfer(to = {:?}, token_id = {:?})",
                to, token_id
            ));

            // carry out the actual transfer
            self.transfer_impl(env.caller(), to, token_id)
        }

        /// Mints a specified amount of new tokens to a given address
        pub(external) fn mint(&mut self, to: AccountId, value: Balance) -> bool {
            env.println(&format!(
                "NFToken::mint(to = {:?}, value = {:?})",
                to, value
            ));
            // carry out the actual minting
            self.mint_impl(env.caller(), value)
        }
    }

    /// Private Methods
    impl NFToken {

        /// Emits a transfer event.
        fn emit_transfer<F, T>(
            from: F,
            to: T,
            token_id: u64,
        )
        where
            F: Into<Option<AccountId>>,
            T: Into<Option<AccountId>>,
        {
            let (from, to) = (from.into(), to.into());
            assert!(from.is_some() || to.is_some());
            assert_ne!(from, to);
            assert!(token_id != 0);
            deposit_event(Event::Transfer { from, to, token_id });
        }

        /// Emits a minting event
        fn emit_mint(
            owner: AccountId,
            value: Balance,
        ) {
            assert!(value > 0);
            deposit_event(Event::Mint { owner, value });
        }

        fn is_token_owner(&self, of: &AccountId, token_id: u64) -> bool {
            let owner = self.id_to_owner.get(&token_id);
            if let None = owner {
                return false;
            }
            let owner = *owner.unwrap();

            env::println(&format!(
                "NFToken:: owner of token id {:?}: {:?}",
                token_id, owner
            ));
            if owner != *of {
                return false;
            }
            true
        }

        /// Transfers token from a specified address to another address.
        fn transfer_impl(&mut self, from: AccountId, to: AccountId, token_id: u64) -> bool {
            env::println(&format!(
                "NFToken::transfer_impl(from = {:?}, to = {:?}, token_id = {:?})",
                from, to, token_id
            ));

            if !self.is_token_owner(&from, token_id) {
                return false;
            }

            env::println(&format!("Ready to make the transfer"));

            self.id_to_owner.insert(token_id, to);

            //update owner token counts
            let from_owner_count = *self.owner_to_token_count.get(&from).unwrap_or(&0);
            let to_owner_count = *self.owner_to_token_count.get(&to).unwrap_or(&0);

            self.owner_to_token_count.insert(from, from_owner_count - 1);
            self.owner_to_token_count.insert(to, to_owner_count + 1);

            Self::emit_transfer(from, to, token_id);
            true
        }

        /// minting of new tokens implementation
        fn mint_impl(&mut self, receiver: AccountId, value: Balance) -> bool {
            env::println(&format!(
                "NFToken::mint_impl(receiver = {:?}, value = {:?})",
                receiver, value
            ));

            let start_id = *self.total_minted + 1;
            let stop_id = *self.total_minted + value;

            // loop through new tokens being minted
            for token_id in start_id..stop_id {
                self.id_to_owner.insert(token_id, receiver);
            }

            // update total supply of owner
            let from_owner_count = *self.owner_to_token_count.get(&self.owner).unwrap_or(&0);
            self.owner_to_token_count.insert(*self.owner, from_owner_count + value);

            // update total supply
            self.total_minted += value;

            Self::emit_mint(receiver, *self.total_minted);
            true
        }
    }
}

#[cfg(all(test, feature = "test-env"))]
mod tests {
    use super::*;
    use std::convert::TryFrom;

    #[test]
    fn deployment() {

        // deploying and minting initial tokens
        let mut _nftoken = NFToken::deploy_mock(100);
        let alice = AccountId::try_from([0x0; 32]).unwrap();
        let bob = AccountId::try_from([0x1; 32]).unwrap();

        let total_minted = _nftoken.total_minted();
        assert_eq!(total_minted, 100);

        // transferring token_id from alice to bob
        _nftoken.transfer(bob, 1);

        let alice_balance = _nftoken.balance_of(alice);
        let bob_balance = _nftoken.balance_of(bob);

        assert_eq!(alice_balance, 99);
        assert_eq!(bob_balance, 1);
    }
}
