use scrypto::prelude::*;

#[blueprint]
mod airdropper {
    enable_method_auth! {
        roles {
            dao => updatable_by: [];
            airdropper => updatable_by: [dao];
        },
        methods {
            airdrop => restrict_to: [dao, airdropper];
            add_airdropper => restrict_to: [dao, airdropper];
            disable_component => restrict_to: [dao];
            input_badge => PUBLIC;
        }
    }

    struct Airdropper {
        airdropper_id_address: ResourceAddress,
        functional: bool,
        locker: Global<AccountLocker>,
        badge_vault: FungibleVault,
    }

    impl Airdropper {
        pub fn drop_it_like_its_hot(
            badge_address: ResourceAddress,
            locker: Global<AccountLocker>,
        ) -> (Global<Airdropper>, Bucket) {
            let (address_reservation, component_address) =
                Runtime::allocate_component_address(Airdropper::blueprint_id());

            let airdropper_ids: Bucket = ResourceBuilder::new_fungible(OwnerRole::None)
                .divisibility(DIVISIBILITY_MAXIMUM)
                .metadata(metadata! (
                    init {
                        "name" => "Certified ILIS Airdropper Badge", updatable;
                        "symbol" => "airdrILIS", updatable;
                        "info_url" => "https://ilikeitstable.com", updatable;
                        "icon_url" => Url::of("https://ilikeitstable.com/images/ilislogo.png"), updatable;
                    }
                ))
                .mint_roles(mint_roles!(
                    minter => rule!(require(global_caller(component_address)));
                    minter_updater => rule!(deny_all);
                ))
                .mint_initial_supply(1)
                .into();

            let airdrop_access_rule: AccessRule = rule!(require(airdropper_ids.resource_address()));

            let component = Self {
                airdropper_id_address: airdropper_ids.resource_address(),
                functional: true,
                locker,
                badge_vault: FungibleVault::new(badge_address),
            }
            .instantiate()
            .prepare_to_globalize(OwnerRole::Fixed(rule!(require(badge_address))))
            .roles(roles! {
                dao => OWNER;
                airdropper => airdrop_access_rule;
            })
            .with_address(address_reservation)
            .globalize();

            (component, airdropper_ids)
        }

        pub fn airdrop(
            &mut self,
            airdrop: Bucket,
            claimants: IndexMap<Global<Account>, ResourceSpecifier>,
        ) -> Option<Bucket> {
            assert!(
                self.functional,
                "not allowed to airdrop anymore, component not functional"
            );
            self.badge_vault
                .authorize_with_amount(dec!("1"), || self.locker.airdrop(claimants, airdrop, true))
        }

        pub fn add_airdropper(&mut self) -> Bucket {
            ResourceManager::from(self.airdropper_id_address).mint(dec!(1))
        }

        pub fn disable_component(&mut self) -> Bucket {
            self.functional = false;
            self.badge_vault.take_all().into()
        }

        pub fn input_badge(&mut self, badge: Bucket) {
            self.badge_vault.put(badge.as_fungible());
        }
    }
}
