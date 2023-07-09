use scrypto::prelude::*;

#[blueprint]
mod myBluePrint {
    struct MyBluePrint {
        my_vault: Vault,
    }

    impl MyBluePrint {
        pub fn instantiate_myBluePrint() -> ComponentAddress {
            let my_bucket: Bucket = ResourceBuilder::new_fungible()
                .metadata("name", "kobin")
                .metadata("symbol", "KOB")
                .mint_initial_supply(1000);

            Self {
                my_vault: Vault::with_bucket(my_bucket),
            }
            .instantiate()
            .globalize()
        }

        pub fn show_Vault(self) {
            info!("My balance is {}", self.my_vault.amount())
        }

        pub fn free_token(&mut self) {
            self.my_vault.take(1)
        }
    }
}
