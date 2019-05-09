# ink! Non Fungible Token

### A bare-bones non-fungible token implemented in ink!

<img src="https://jkrb.co/misc/nftoken_ink.png" width="400" />

[Part 1 now published](https://medium.com/block-journal/introducing-substrate-smart-contracts-with-ink-d486289e2b59) @ The Block Journal on Medium

[Part 2 now published](https://medium.com/@rossbulat/writing-a-substrate-smart-contract-with-ink-1f178849f931) @ The Block Journal on Medium

[Part 3 now published](https://medium.com/block-journal/deploying-an-ink-smart-contract-to-a-substrate-chain-f52a2a36efec) @ The Block Journal on Medium

### ink!: Substrate’s smart contract language

Parity’s Substrate blockchain framework, of which Polkadot is being built on top of, is in active development and making rapid progress towards a final release. Ink (or ink! as the name is commonly termed in documentation) is Parity’s solution to writing smart contracts for a Substrate based blockchain.

Like Substrate, Ink is built on top of Rust, and therefore adheres to Rust language rules and syntax. This tutorial will walk through an example smart contract replicating a non-fungible token, commonly referred to as ERC721 tokens on the Ethereum blockchain. Specifically, the contract resembles a non-fungible token contract that will support 3 main features: minting tokens, transferring tokens, and approving another account to send tokens on your behalf.

### A note on non-fungible tokens

Non-fungible tokens, or NFTs, differ from ERC20 style tokens whereby every token is unique. In our contract, each token will have a unique ID used to represent the token. From here, each token could have its own value, its own metadata, or have a specific use within an application or value system.

Approving non-owner accounts to transfer or manage the token is also different, and has to be done on a per-token basis with Non-fungible tokens. Cryptokitties is the best known example where Non-fungible tokens have been implemented — each token representing a kitten on the platform.

NFTs present different challenges to a standard token, and therefore give us more to explore in terms of Ink syntax and capabilities.

## Setting up the Ink Project

The easiest way to bootstrap an Ink project currently is to install the "Hello World" contract of Ink, named Flipper. With Flipper installed, we can build upon what is already included and not have to worry about configuration and compile scripts — these are provided in Flipper.

Note: Both Substrate and Ink are in rapid development and are not yet feature complete, therefore the smart contract environment, and the smart contract code itself, will most likely change as Parity get nearer to a final release of the framework.

To jump start our Ink project fetch Flipper using cargo:

```
# fetch the Flipper Ink contract

cargo contract new flipper
```

Flipper provides us a project boilerplate needed to start writing the smart contract. Included is:

The folder structure and configuration metadata of the project
A bare-bones Flipper contract in src/lib.rs, that simply “flips” a boolean value between true and false via a flip() method, and gets this value on-chain using the get() method. We will be replacing this file with the NFT contract
The Rust specific Cargo.toml file, outlining the project dependencies and module metadata, a .gitignore file, and a build.sh file. The build.sh file is what we run to compile our smart contract, resulting in a compiled .wasm file of the contract, a JSON abstraction of the contract, and more. We’ll explore the built contract further down.

*__Note__: Now is a good time to check out src/lib.rs to get a feel of the contract syntax.*

Let’s change the name flipper to a more suitable name: nftoken. Amend the following:

* `flipper/` folder name to /nftoken
* `Cargo.tom`l: Change `[package] name` and `[lib] name` to `nftoken`
* `build.sh`: amend `PROJNAME=nftoken`

Also, ensure we have permissions to run `nftoken/build.sh`:
```
cd nftoken
chmod +x build.sh
```
Lastly, add the /nftoken folder to a VS Code Workspace, and we are ready to start writing.

## About Ink
Ink has [multiple levels](https://github.com/paritytech/ink#structure) of abstraction, where higher levels abstract over the lower levels. We will be using the highest level, which is dubbed the language level, or `lang` level. These levels have also been separated into modules that can be explored [here](https://paritytech.github.io/ink/pdsl_core/index.html).

Below the `lang` module are the `model` and `core` modules, that focus on mid-level abstractions and core utilities respectively. Below the `core` module we can also expect a CLI specifically for creating and managing Ink contracts.

Although there is little coverage on how to use these modules at the time of writing, we do indeed have the raw API docs to browse through, both for the [core](https://paritytech.github.io/ink/pdsl_core/index.html) module and [model](https://paritytech.github.io/ink/pdsl_model/index.html) module. If you are following this article these docs can be browsed through now, although our contract below will utilise some of these APIs intended to show how they are used in the context of the `lang` level via the non-fungible token contract.

With this in mind, let’s next examine what the structure of our lang derived contract looks like, and compare it to what we expect from a Solidity based smart contract.

## Contract Structure

Structuring an Ink contract is similar to that of a Solidity contract, where the major components we have come to expect with Solidity are also consistent in Ink: contract variables, events, public functions and private functions, as well as environment variables to grab the caller address and more.

Below is an abstraction of how the NFToken contract is structured:

```
// declare modules
use parity::<module>
...

//define events
enum Event {
   Mint { owner: AccountId, value: u64 },
}
...

//wrap entire contract inside the contract! macro
contract! {
   
   // contract variables as a struct
   struct NFToken {
      owner: storage::Value<AccountId>,
      ...
   }
   
   // compulsory deploy method that is run upon the initial contract instantiation
   impl Deploy for NFToken {
      fn deploy(&mut self, init_value: u64){}
   }
   
   // public contract methods in an impl{} block
   impl NFToken {
      pub(external) fn total_minted(&self) -> u64 {}
      ...
   }
   
   // private contract methods in a separate impl{} block
   imp NFToken {
      fn is_token_owner(
         &self, 
         of: &AccountId, 
         token_id: u64) -> bool {}
       ...
   }
}

// test functions
mod tests {
   fn it_works() {}
   ...
}
```

Let’s briefly visit these sections and how they differ from what we have come to expect from a Solidity contract. Ink is built upon Rust, so all the syntax here is valid Rust syntax.

* Our module declaration section is where we bring external functionality into the contract, and is the similar in nature to Solidity’s `using` declarations.

```
// Ink
use ink_core::{
    env::{self, AccountId},
    memory::format,
    storage,
};
use ink_lang::contract;
use parity_codec::{Decode, Encode};

// Solidity
interface ContractName {
   using SafeMath for uint256;
   using AddressUtils for address;
}
```
* Events are declared inside an `Event` enum, whereas with Solidity we define our events separately, typing each as an `event`:

```
// Ink
enum Event {
   Transfer { 
      from: AccountId, 
      to: AccountId, 
      token_id: u64 
   },
}

// Solidity
event Transfer(
   address indexed from,
   address indexed to,
   uint256 indexed _tokenId
);
```
* Where a Solidity contract is embedded within an `interface` block, an Ink contract is embedded within a `contract!` macro. Our events are declared outside of this macro, whereas events are declared within a Solidity interface. This is described below.

*__Note__: A [macro](https://doc.rust-lang.org/book/macros.html) in Rust is a a declaration that represents a block of syntax that the wrapped expressions will be surrounded by. Macros abstract at a syntactic level, so the `contract!` macro is wrapping its contents with more syntax.*

```
// Ink

// events
contract! {
   // rest of contract
}

// Solidity

interface ContractName {
   // events
   // rest of contract
}
```

* With Ink, our contract variables are written in a struct of the name of our contract. Hash maps derived from Rust’s `HashMap` type are in place of Solidity’s `mapping` type, providing `key => value` lists.

### How Substrate stores values

Any piece of data persisted on a Substrate chain is called an *extrinsic*, and Ink provides us the means to store extrinsics on-chain via the `storage` module, that lives within the `core` module of the language. In other words, all contract variables that you plan to persist on chain will use a type from `storage`. Conversely, the `memory` module is also available for data structures to operate on memory.

Solidity on the other hand adopts a different approach to this. From Solidity 0.5, `storage` and `memory` reference types were introduced to function arguments or function variables, so the contract knows where to reference those variables. However, this is not necessary for contract variables in Solidity.

Primitive types are also available and consistent throughout both languages; where Rust uses `u64`, Solidity adopts a more verbose `uint64` type declaration. Overall it is quite simple to achieve the same contract variable structure between the two languages.

```
// Ink
struct NFToken {
   owner: storage::Value<AccountId>,
   approvals: storage::HashMap<u64, AccountId>,
}

// Solidity
address owner;
mapping (uint64 => address) approvals;
```

In the above example, the type of values that `storage` objects handle are passed in via the angled brackets in place of the type’s generics.

* The concept of an initialisation function is present within both Ink and Solidity, albeit implemented in different ways. With Ink, we explicitly define a `deploy()` method within a `Deploy{}` implementation block. The parameters we define for this method are representative of what data we send when initialising the contract. E.g. for our non-fungible token, we will provide an initial amount of tokens to be minted:

```
// Inks initialisation function, deploy()
impl Deploy for NFToken {
   fn deploy(&mut self, init_value: u64) {
      ...
   }
 }
 ```
 
 * Public and private methods are also defined within `impl` blocks, where public methods are explicitly defined with `pub(external)`. Again, when comparing this syntax to that of Solidity’s, `internal` and `external` are used to define a private or public resource.

*__Note__: In Rust, functions, modules, traits and structs are private by default, and must be defined with the `pub` keyword in order for them to be externally reachable. The `(external)` extension to `pub` here is Ink specific, and is compulsory to include with public Ink functions.*

```
// public functions
impl NFToken {
   pub(external) fn total_minted(&self) -> u64 {}
}
// private functions
impl NFToken {
   fn mint_impl(
      &mut self, 
      receiver: AccountId, 
      value: u64) -> bool { 
    }
}
```

Again, we have separated our private and public functions in separate `impl` blocks, and have included `pub(external)` for public functions defined.

As the last building block of our contract, a `tests` module is defined that asserts various conditions as our functions are tested. Within the `tests` module, we can test our contract logic without having to compile and deploy it to a Substrate chain, allowing speedy ironing out of bugs and verification that the contract works as expected.





















