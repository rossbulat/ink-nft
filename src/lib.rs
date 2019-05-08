// Copyright 2017-2019 JKRB Investments Limited.
//
// You should have received a copy of the GNU General Public License
// along with this file.  If not, see <http://www.gnu.org/licenses/>.

#![cfg_attr(not(any(test, feature = "std")), no_std)]

use ink_core::{
    env::{self, println, AccountId, Balance},
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
    /// Emits when an approved address for an NFT is changed or re-affirmed.
    Approval {
        owner: AccountId,
        spender: AccountId,
        token_id: u64,
        approved: bool,
    },
}

/// Deposits an NFToken event.
fn deposit_event(event: Event) {
    env::deposit_raw_event(&event.encode()[..])
}

contract! {
    /// Storage values of the contract
    struct NFToken {
        /// Owner of contract
        owner: storage::Value<AccountId>,
        /// Total tokens minted
        total_minted: storage::Value<u64>,
        /// Mapping: token_id(u64) -> owner (AccountID)
        id_to_owner: storage::HashMap<u64, AccountId>,
        /// Mapping: owner(AccountID) -> tokenCount (Balance)
        owner_to_token_count: storage::HashMap<AccountId, Balance>,
        /// Mapping: token_id(u64) to account(AccountId)
        approvals: storage::HashMap<u64, AccountId>,
    }

    /// compulsary Demploy method
    impl Deploy for NFToken {
        /// Initializes our initial total minted value to 0.
        fn deploy(&mut self, init_value: u64) {
            self.total_minted.set(0);
            // set ownership of contract
            self.owner.set(env.caller());
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
            println(&format!("NFToken::total_minted = {:?}", total_minted));
            total_minted
        }

        /// Return the balance of the given address.
        pub(external) fn balance_of(&self, owner: AccountId) -> Balance {
            let balance = *self.owner_to_token_count.get(&owner).unwrap_or(&0);
            println(&format!("NFToken::balance_of(owner = {:?}) = {:?}", owner, balance));
            balance
        }

        /// Transfers a token_id to a specified address from the caller
        pub(external) fn transfer(&mut self, to: AccountId, token_id: u64) -> bool {
            println(&format!(
                "NFToken::transfer(to = {:?}, token_id = {:?})",
                to, token_id
            ));

            // carry out the actual transfer
            self.transfer_impl(env.caller(), to, token_id)
        }

        /// Transfers a token_id from a specified address to another specified address
        pub(external) fn transfer_from(&mut self, to: AccountId, token_id: u64) -> bool {
            println(&format!(
                "NFToken::transfer_from(from = {:?}, to = {:?}, token_id = {:?})",
                env.caller(), to, token_id
            ));

            // make the transfer immediately if caller is the owner
            if self.is_token_owner(&env.caller(), token_id) {
                println(&format!("approval: Caller is the owner, send immdeiately"));
                let result = self.transfer_impl(env.caller(), to, token_id);
                return result;

            // not owner: check if caller is approved to move the token
            } else {

                println(&format!("approval: Caller is not the owner, needs to be approved."));
                let approval = self.approvals.get(&token_id);
                if let None = approval {
                    println(&format!("approval: No approvals exist, returning now."));
                    return false;
                }

                //carry out transfer if caller is approved
                if *approval.unwrap() == env.caller() {
                    println(&format!("approval: Found approval is the caller - make transfer"));
                    // carry out the actual transfer
                    let result = self.transfer_impl(env.caller(), to, token_id);
                    return result;
                } else {

                    println(&format!("approval: Found approval is not the caller - returning now"));
                    return false;
                }
            }
        }

        /// Mints a specified amount of new tokens to a given address
        pub(external) fn mint(&mut self, to: AccountId, value: u64) -> bool {
            println(&format!(
                "NFToken::mint(to = {:?}, value = {:?})",
                to, value
            ));
            // carry out the actual minting
            self.mint_impl(env.caller(), value)
        }

         /// Approves or disapproves an Account to send token on behalf of an owner
        pub(external) fn approval(&mut self, to: AccountId, token_id: u64, approved: bool) -> bool {
            println(&format!(
                "NFToken::approval(account = {:?}, token_id: {:?}, approved = {:?})",
                to, token_id, approved
            ));

            // return if caller is not the token owner
            let token_owner = self.id_to_owner.get(&token_id);
            if let None = token_owner {
                println(&format!("approval: Could not find token owner"));
                return false;
            }

            let token_owner = *token_owner.unwrap();
            if token_owner != env.caller() {
                println(&format!("approval: Not token owner"));
                return false;
            }

            let approvals = self.approvals.get(&token_id);

            // insert approval if
            if let None = approvals {
                if approved == true {
                    println(&format!("approval: Approval does not exist. Inserting now"));
                    self.approvals.insert(token_id, to);
                } else {
                    println(&format!("NFToken::approval: Approval does not exist. nothing to remove"));
                    return false;
                }

            } else {
                let existing = *approvals.unwrap();

                // remove existing owner if disapproving
                if existing == to && approved == false {
                    println(&format!("approval: Approved account exists. Removing now"));
                    self.approvals.remove(&token_id);
                }

                // overwrite or insert if approving is true
                if approved == true {
                    println(&format!("approval: Inserting or overwriting approval"));
                    self.approvals.insert(token_id, to);
                }
            }

            println(&format!("approval: Emitting approval event"));
            Self::emit_approval(&self, env.caller(), to, token_id, approved);
            true
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

        /// Emits an approval event.
        fn emit_approval(
            &self,
            owner: AccountId,
            spender: AccountId,
            token_id: u64,
            approved: bool,
        ) {
            assert_ne!(owner, spender);
            assert!(token_id > 0);
            deposit_event(Event::Approval { owner, spender, token_id, approved });
        }

        fn is_token_owner(&self, of: &AccountId, token_id: u64) -> bool {
            let owner = self.id_to_owner.get(&token_id);
            if let None = owner {
                return false;
            }
            let owner = *owner.unwrap();

            println(&format!(
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
        fn mint_impl(&mut self, receiver: AccountId, value: u64) -> bool {
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
    fn it_works() {

        // deploying and minting initial tokens
        let mut _nftoken = NFToken::deploy_mock(100);
        let alice = AccountId::try_from([0x0; 32]).unwrap();
        let bob = AccountId::try_from([0x1; 32]).unwrap();
        let charlie = AccountId::try_from([0x2; 32]).unwrap();
        let dave = AccountId::try_from([0x3; 32]).unwrap();

        let total_minted = _nftoken.total_minted();
        assert_eq!(total_minted, 100);

        // transferring token_id from alice to bob
        _nftoken.transfer(bob, 1);

        let alice_balance = _nftoken.balance_of(alice);
        let mut bob_balance = _nftoken.balance_of(bob);

        assert_eq!(alice_balance, 99);
        assert_eq!(bob_balance, 1);

        // approve charlie to send token_id 2 from alice's account
        _nftoken.approval(charlie, 2, true);

        // get_token_approval()
        // assert result

        // overwrite charlie's approval with dave's approval
        _nftoken.approval(dave, 2, true);

        // get_token_approval()
        // assert result

        // remove dave from approvals
        _nftoken.approval(dave, 2, false);

        // get_token_approval()
        // assert result

        // transfer_from function: caller is token owner
        _nftoken.approval(charlie, 3, true);
        _nftoken.transfer_from(bob, 3);

        bob_balance = _nftoken.balance_of(bob);
        assert_eq!(bob_balance, 2);
    }
}
