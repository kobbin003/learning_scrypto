use scrypto::prelude::*;

#[derive(ScryptoSbor)]
enum Color {
    Red,
    Blue,
    Green,
}
#[derive(ScryptoSbor)]
enum Rarity {
    Common,
    Uncommon,
    Rare,
    Legendary,
}
#[derive(NonFungibleData, ScryptoSbor)]

struct Nft {
    color: Color,
    rarity: Rarity,
    #[mutable]
    level: u8,
}
#[blueprint]
mod mutable_nft {

    struct MutableNft {
        nft_mint_badge: Vault,
        nft_resource_address: ResourceAddress,
        nft_price: u64,
        collected_xrd: Vault,
        nft_id_counter: u64,
    }

    impl MutableNft {
        pub fn instantiate_mutable_nft() -> ComponentAddress {
            // create badge(badges are required for authorization)
            let nft_mint_badge: Bucket = ResourceBuilder::new_fungible()
                .divisibility(DIVISIBILITY_NONE)
                .metadata("name", "nft Mint Badge")
                .mint_initial_supply(1);

            // the nft item
            let nft_resource_address: ResourceAddress =
                ResourceBuilder::new_integer_non_fungible::<Nft>()
                    .metadata("name", "mutablenft")
                    .metadata("symbol", "MNFT")
                    .mintable(rule!(require(nft_mint_badge.resource_address())), LOCKED)
                    .burnable(rule!(require(nft_mint_badge.resource_address())), LOCKED)
                    .updateable_non_fungible_data(
                        rule!(require(nft_mint_badge.resource_address())),
                        LOCKED,
                    )
                    .create_with_no_initial_supply();
            Self {
                nft_mint_badge: Vault::with_bucket(nft_mint_badge),
                nft_resource_address,
                nft_price: 50,
                nft_id_counter: 0,
                collected_xrd: Vault::new(RADIX_TOKEN),
            }
            .instantiate()
            .globalize()
        }

        /*//! HOW TO CALL buy_nft method() with the rdx token as argument???
        //! resim call-method [comp_adr] [method]*/
        //* resim call-method component_sim1qdms4etzwnhsdpj5ys9shy9jgv2yp0z9w6ud8z9fyv7sksuzxn buy_nft resource_sim1qyqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqs6d89k:50
        pub fn buy_nft(&mut self, mut payment: Bucket) -> (Bucket, Bucket) {
            // PUT the payment into the collected_xrd vault
            // CHECK if the buyer has sent XRD token
            assert_eq!(
                self.collected_xrd.resource_address(),
                payment.resource_address(),
                "The token you sent is incorrect. Please send a XRD token only!"
            );

            // put the payed xrd into collected_xrd Vault.
            self.collected_xrd.put(payment.take(self.nft_price));

            //mint a new nft

            let nft_item = Nft {
                color: Color::Red,
                rarity: Rarity::Common,
                level: 1,
            };

            // authorize() is called on "nft_mint_badge"
            //which has the right to mint, burn & update the nft
            let minted_nft = self.nft_mint_badge.authorize(|| {
                // use "borrrow_resource_manager" to access the resource manager
                let resource_manager = borrow_resource_manager!(self.nft_resource_address);

                resource_manager
                    .mint_non_fungible(&NonFungibleLocalId::integer(self.nft_id_counter), nft_item)
            });
            // increment the counter
            self.nft_id_counter += 1;
            // Return the NFT and change
            (minted_nft, payment)
        }

        pub fn show_token_info(&self) {
            // We borrow the resource manager of the provided address
            let manager: ResourceManager = borrow_resource_manager!(self.nft_resource_address);

            // Get the resource type
            match manager.resource_type() {
                ResourceType::Fungible { divisibility } => {
                    info!("Fungible resource with divisibility of {}", divisibility)
                }
                ResourceType::NonFungible { id_type } => {
                    info!("Non Fungible resource")
                }
            }

            // Get the total supply
            info!("Total supply: {}", manager.total_supply());

            // Get information stored in the metadata
            let metadata = manager.metadata();

            // let metadata: HashMap<String, String> = manager.metadata();
            // let token_name = metadata.get("name").expect("Token does not have a name");
            // let token_symbol = metadata
            //     .get("symbol")
            //     .expect("Token does not have a symbol");
            // info!("Name: {}. Symbol: {}", token_name, token_symbol);
            // info!("MetaData: {}", metadata);
        }
    }
}
