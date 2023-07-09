use scrypto::prelude::*;
#[derive(NonFungibleData, ScryptoSbor)]
struct NftItem {
    #[mutable]
    xrd_claimed: bool,
}

#[blueprint]
mod xrd_prize {
    struct XrdPrize {
        nft_resource: ResourceAddress,
        admin_badge: Vault,
        funds_vault: Vault,
    }

    impl XrdPrize {
        // funds in XRD
        pub fn instantiate_xrd_prize(funds: Bucket) -> ComponentAddress {
            let admin_badge: Bucket = ResourceBuilder::new_fungible().mint_initial_supply(1);

            let my_nft_data_resource_address: ResourceAddress =
                ResourceBuilder::new_uuid_non_fungible::<NftItem>()
                    .metadata("name", "MyNFTData")
                    .metadata("symbol", "MND")
                    .mintable(rule!(require(admin_badge.resource_address())), LOCKED)
                    .updateable_non_fungible_data(
                        rule!(require(admin_badge.resource_address())),
                        LOCKED,
                    )
                    .create_with_no_initial_supply();
            Self {
                nft_resource: my_nft_data_resource_address,
                admin_badge: Vault::with_bucket(admin_badge),
                funds_vault: Vault::with_bucket(funds),
            }
            .instantiate()
            .globalize()
        }

        pub fn mint_nft(&mut self) -> Bucket {
            // let admin_badge = &self.admin_badge;
            let minted_nft = self.admin_badge.authorize(|| {
                let resource_manager = self::borrow_resource_manager!(self.nft_resource);
                resource_manager.mint_uuid_non_fungible(NftItem { xrd_claimed: false })
            });
            return minted_nft;
        }

        // PASSING PROOF BY INTENT :
        // this mode of authentication is used for methods which requires the NonFungibleData of the Proof
        pub fn claim_xrd(&mut self, auth: Proof) -> Bucket {
            //check if the proof is a MyNFTData
            let auth = auth
                .validate_proof(self.nft_resource)
                .expect("invalid nft provided");

            //get the non-fungible from the proof
            //* However, note that we can only view and utilize the NonFungibleData of the Proof after we’ve validated the proof!
            let nft: NonFungible<NftItem> = auth.non_fungible();

            // get the data of the non-fungible
            let nft_data: NftItem = nft.data();
            info!("nft_data: {}", nft_data.xrd_claimed);
            // if xrd_claimed = false-> PASS or SHOW ERROR-MESSAGE
            // assert!(!nft_data.xrd_claimed, "xrd already claimed");

            // update the xrd_claim field to true.
            self.admin_badge.authorize(|| {
                self::borrow_resource_manager!(self.nft_resource).update_non_fungible_data(
                    nft.local_id(),
                    "xrd_claimed",
                    true,
                )
            });
            // take xrd from fund
            self.funds_vault.take(10)
        }
    }
}
