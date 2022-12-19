//! PSP22 and ERC20 EVM contract interoperability using XVM interface.
#![cfg_attr(not(feature = "std"), no_std)]

pub use self::xvm_transfer::{
    XvmTransfer,
    XvmTransferRef,
};
use ink_lang as ink;

/// EVM ID (from astar runtime)
const EVM_ID: u8 = 0x0F;

#[ink::contract(env = xvm_environment::XvmDefaultEnvironment)]
mod xvm_transfer {
    use ethabi::{
        ethereum_types::{
            H160,
            U256,
        },
        Token,
    };
    use hex_literal::hex;
    use ink_prelude::{
        string::{
            String,
            ToString,
        },
        vec::Vec,
    };

    const TRANSFER_SELECTOR: [u8; 4] = hex!["a9059cbb"];

    /// Only one Error is supported
    #[derive(Debug, scale::Encode, scale::Decode)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub enum PSP22Error {
        Custom(String),
    }

    #[ink(storage)]
    pub struct XvmTransfer {}

    impl XvmTransfer {
        #[ink(constructor)]
        pub fn new() -> Self {
            Self {}
        }

        // #[ink(message)]
        pub fn transfer_to_evm(&mut self, to: [u8; 20], value: u128, erc20_address: [u8; 20]) -> bool {
            let encoded_input = Self::transfer_encode(to.into(), value.into());
            self.env()
                .extension()
                .xvm_call(
                    super::EVM_ID,
                    Vec::from(erc20_address.as_ref()),
                    encoded_input,
                )
                .is_ok()
        }

        #[ink(message, selector = 0xdb20f9f5)]
        pub fn transfer_to_native(
            &mut self,
            to: AccountId,
            value: Balance,
            erc20_address: [u8; 20],
            _data: Vec<u8>,
        ) -> Result<(), PSP22Error> {
            let encoded_input = Self::transfer_encode(Self::h160(&to), value.into());
            self.env()
                .extension()
                .xvm_call(
                    super::EVM_ID,
                    Vec::from(erc20_address.as_ref()),
                    encoded_input,
                )
                .map_err(|_| PSP22Error::Custom(String::from("transfer failed")))
        }

        // Todos:
        //   1. modify the type anotaion for `to` parameter to receive both AccountId([u8; 32]) and [u8; 20] 
        //   2. make a function that returns 'is_send_to_evm_account' boolean variable (or I think we can add this into parameter)
        #[ink(message)]
        pub fn transfer(
            &mut self,
            to: AccountId,
            value: Balance,
            erc20_address: [u8; 20],
            // is_send_to_evm: bool,
            _data: Vec<u8>,
        ) -> Result<(), PSP22Error> {
            let is_send_to_evm_account = true;

            if is_send_to_evm_account {
                Self::transfer_to_evm(self, to, value, erc20_address, _data)
            } else {
                Self::transfer_to_native(self, to, value, erc20_address, _data)
            }
        }


        /// Helper function to get H160 address of the 32 bytes accountId
        #[ink(message)]
        pub fn to_h160_address(&self, from: AccountId) -> String {
            Self::h160(&from).to_string()
        }

        fn transfer_encode(to: H160, value: U256) -> Vec<u8> {
            let mut encoded = TRANSFER_SELECTOR.to_vec();
            let input = [Token::Address(to), Token::Uint(value)];
            encoded.extend(&ethabi::encode(&input));
            encoded
        }

        fn h160(from: &AccountId) -> H160 {
            let mut dest: H160 = [0; 20].into();
            dest.as_bytes_mut()
                .copy_from_slice(&<ink_env::AccountId as AsRef<[u8]>>::as_ref(from)[..20]);
            dest
        }
    }
}
