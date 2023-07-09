use scrypto::prelude::*;

#[blueprint]
mod token_sale {
    struct TokenSale {
        ust_vault: Vault,
        xrd_reserve: Vault,
        token_price: u64,
    }

    impl TokenSale {
        pub fn instantiate_token_sale() -> ComponentAddress {
            let token: Bucket = ResourceBuilder::new_fungible()
                .metadata("name", "useful token")
                .metadata("symbol", "UST")
                .divisibility(DIVISIBILITY_NONE)
                .mintable(AccessRule::AllowAll, LOCKED)
                .mint_initial_supply(10);
            Self {
                ust_vault: Vault::with_bucket(token),
                xrd_reserve: Vault::new(RADIX_TOKEN),
                token_price: 10,
            }
            .instantiate()
            .globalize()
        }

        pub fn buy_token(&mut self, mut payment: Bucket) -> (Bucket, Bucket) {
            // check if xrd is payed
            assert_eq!(
                self.xrd_reserve.resource_address(),
                payment.resource_address(),
                "The token you sent is incorrect. Please send a XRD token only!"
            );

            let num_of_token = payment.amount() / self.token_price;
            let change = payment.amount() - (num_of_token * self.token_price);

            // take the paid amount
            self.xrd_reserve
                .put(payment.take(payment.amount() - change));

            let manager: ResourceManager =
                borrow_resource_manager!(self.ust_vault.resource_address());
            /* //! mint token if token.amount is empty */
            let tokens;
            if self.ust_vault.amount() < num_of_token {
                tokens = manager.mint(num_of_token);
            } else {
                tokens = self.ust_vault.take(num_of_token);
            }

            // return the tokens and the remaining XRD
            (tokens, payment)
        }

        pub fn show_token(&mut self, payment: Bucket) {
            debug!("payment address: {:?}", payment.resource_address());
            debug!("XRD address: {:?}", self.xrd_reserve.resource_address());
        }

        pub fn sell(&mut self, token_to_sell: Bucket, token_price: u64) -> Bucket {
            //check if the token is the useful token
            assert_eq!(
                self.ust_vault.resource_address(),
                token_to_sell.resource_address(),
                "you can only sell useful token(UST)"
            );

            let xrd = token_to_sell.amount() * token_price;
            self.ust_vault.put(token_to_sell);
            //return the XRD
            self.xrd_reserve.take(xrd)
        }
    }
}
