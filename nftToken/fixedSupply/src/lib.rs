use scrypto::prelude::*;
#[derive(ScryptoSbor)]
pub enum Color {
    White,
    Blue,
    Black,
    Red,
    Green,
}

#[derive(ScryptoSbor)]
pub enum Rarity {
    Common,
    Uncommon,
    Rare,
    MythicRare,
}
#[derive(ScryptoSbor, NonFungibleData)]

pub struct NftItems {
    color: Color,
    rarity: Rarity,
    #[mutable]
    level: u8,
}

#[blueprint]
mod nft_blue_print {
    struct NftBluePrint {
        // Define what resources and data will be managed by Hello components
        nft_vault: Vault,
    }

    impl NftBluePrint {
        // Implement the functions and methods which will manage those resources and data

        // This is a function, and can be called directly on the blueprint once deployed
        pub fn instantiate_nft_blueprint() -> ComponentAddress {
            // Create a new token called "HelloToken," with a fixed supply of 1000, and put that supply into a bucket
            let nft_bucket: Bucket = ResourceBuilder::new_integer_non_fungible::<NftItems>()
                .metadata("name", "MyNftToken")
                .metadata("symbol", "MNTT")
                .mint_initial_supply([
                    (
                        IntegerNonFungibleLocalId::new(1u64),
                        NftItems {
                            color: Color::Black,
                            rarity: Rarity::MythicRare,
                            level: 3,
                        },
                    ),
                    (
                        IntegerNonFungibleLocalId::new(2u64),
                        NftItems {
                            color: Color::Red,
                            rarity: Rarity::Uncommon,
                            level: 1,
                        },
                    ),
                    (
                        IntegerNonFungibleLocalId::new(3u64),
                        NftItems {
                            color: Color::White,
                            rarity: Rarity::Rare,
                            level: 2,
                        },
                    ),
                ]);

            // Instantiate a Hello component, populating its vault with our supply of 1000 HelloToken
            Self {
                nft_vault: Vault::with_bucket(nft_bucket),
            }
            .instantiate()
            .globalize()
        }

        // This is a method, because it needs a reference to self.  Methods can only be called on components
        pub fn list_nftoken(&mut self) -> Bucket {
            info!("My nft: {:?}", self.nft_vault);
            self.nft_vault.take_all()
        }
    }
}
