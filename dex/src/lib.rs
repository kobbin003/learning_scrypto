use scrypto::prelude::*;

#[blueprint]
mod dex {

    struct Dex {
        pool_a: Vault,
        pool_b: Vault,
        pool_lpt: Vault,
        lpt_mint_badge: Vault,
        lp_per_asset_ratio: Decimal,
        swap_fee: Decimal,
    }

    impl Dex {
        pub fn instantiate_dex(
            a_tokens: Bucket,
            b_tokens: Bucket,
            lp_initial_supply: Decimal,
            lp_symbol: String,
            lp_name: String,
            lp_url: String,
            swap_fee: Decimal,
        ) -> ComponentAddress {
            assert!(
                !(a_tokens.is_empty()) || !(b_tokens.is_empty()),
                "Initial amount of a_tokens and b_tokens should be given."
            );
            assert!(swap_fee >= dec!(0) && swap_fee <= dec!(1), "invalid fee");

            // mint_badge_for_lp_token
            let lpt_mint_badge: Bucket = ResourceBuilder::new_fungible().mint_initial_supply(1);

            // liquidity token
            let lpt_addr: ResourceAddress = ResourceBuilder::new_fungible()
                .metadata("name", lp_name)
                .metadata("symbol", lp_symbol)
                .metadata("url", lp_url)
                .mintable(rule!(require(lpt_mint_badge.resource_address())), LOCKED)
                .create_with_no_initial_supply();

            // mint lp_token based on the given lp_initial_supply
            let lp_tokens = lpt_mint_badge
                .authorize(|| borrow_resource_manager!(lpt_addr).mint(lp_initial_supply));

            let lp_per_asset_ratio = lp_initial_supply / (a_tokens.amount() * b_tokens.amount());

            Self {
                pool_a: Vault::with_bucket(a_tokens),
                pool_b: Vault::with_bucket(b_tokens),
                pool_lpt: Vault::with_bucket(lp_tokens),
                lpt_mint_badge: Vault::with_bucket(lpt_mint_badge),
                lp_per_asset_ratio,
                swap_fee,
            }
            .instantiate()
            .globalize()
        }

        pub fn swap(&mut self, input_token: Bucket) -> Bucket {
            let (input_token_pool, output_token_pool): (&mut Vault, &mut Vault) =
                if input_token.resource_address() == self.pool_a.resource_address() {
                    (&mut self.pool_a, &mut self.pool_b)
                } else if input_token.resource_address() == self.pool_b.resource_address() {
                    (&mut self.pool_b, &mut self.pool_a)
                } else {
                    panic!("token not of this liquidity pool")
                };

            //calculation
            let output_token_amount =
                (output_token_pool.amount() * self.swap_fee * input_token.amount())
                    / (input_token_pool.amount() + self.swap_fee * input_token.amount());

            //check if liquidity is available for swapping
            if output_token_amount > output_token_pool.amount() {
                panic!("not enough liquidity to swap")
            };

            // take in the input_token
            input_token_pool.put(input_token);

            //give out the output token
            output_token_pool.take(output_token_amount)
        }

        pub fn add_liquidity(
            &mut self,
            mut a_tokens: Bucket,
            mut b_tokens: Bucket,
        ) -> (Bucket, Bucket, Bucket) {
            assert!(
                ((a_tokens.resource_address() == (self.pool_a.resource_address())
                    || (a_tokens.resource_address() == self.pool_b.resource_address()))
                    || (b_tokens.resource_address() == (self.pool_a.resource_address())
                        || (b_tokens.resource_address() == self.pool_b.resource_address()))),
                "tokens not of this liquidity pool!"
            );
            // ADDING LIQUIDITY

            let (a, b) = (self.pool_a.amount(), self.pool_b.amount());

            let (da, db) = (a_tokens.amount(), b_tokens.amount());

            let (da_ratio, db_ratio) = (da / a, db / b);

            let (a_tokens_taken, b_tokens_taken, lpt_required) = if (da / db) == (a / b) {
                (da, db, self.lp_per_asset_ratio * da * db)
            } else if da_ratio > db_ratio {
                //* multiply by minimum of two:
                //  a_tokens_taken = a * db_ratio
                //  b_tokens_taken = b * db_ratio = b * (db/b) = db
                (a * db_ratio, db, self.pool_lpt.amount() * db_ratio)
            } else {
                // if da_ratio < db_ratio
                (da, b * da_ratio, self.pool_lpt.amount() * da_ratio)
            };

            self.pool_a.put(a_tokens.take(a_tokens_taken));
            self.pool_b.put(b_tokens.take(b_tokens_taken));

            // MINT AND GIVEOUT LPT
            let resource_manager = self::borrow_resource_manager!(self.pool_lpt.resource_address());

            // mint lpt
            let minted_lpt = self
                .lpt_mint_badge
                .authorize(|| resource_manager.mint(lpt_required));

            // return
            (a_tokens, b_tokens, minted_lpt)
        }

        pub fn remove_liquidity(&mut self, lp_tokens: Bucket) -> (Bucket, Bucket) {
            assert!(
                lp_tokens.resource_address() == self.pool_lpt.resource_address(),
                "this is not the liquidity provider token of this liquidity pool!"
            );
            //  lp_token_percentage = lp_tokens / self.pool_lpt
            let lp_tokens_share = lp_tokens.amount() / self.pool_lpt.amount();
            // amount of a = lp_token_percentage * self.pool_a.amount()
            let a_tokens_out = lp_tokens_share * self.pool_a.amount();
            // amount of b = lp_token_percentage * self.pool_b.amount()
            let b_tokens_out = lp_tokens_share * self.pool_b.amount();

            // put the lp_tokens in the Vault
            lp_tokens.burn();

            // give out token_a & token_b as per the ratio.
            (
                self.pool_a.take(a_tokens_out),
                self.pool_b.take(b_tokens_out),
            )
        }
    }
}
