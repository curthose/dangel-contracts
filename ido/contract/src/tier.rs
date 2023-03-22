use crate::*;

pub type TierConfigsType = HashMap<Tier, TierConfig>;
/// see https://docs.dangel.fund/h/tiers.
#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize, PartialEq, Debug, Clone, Eq, Hash, PartialOrd, Copy)]
#[serde(crate = "near_sdk::serde")]
pub enum Tier {
    Tier0,
    Tier1,
    Tier2,
    Tier3,
    Tier4,
    Tier5,
    Tier6,
    Tier7,
}
#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Clone, Debug, Copy)]
#[serde(crate = "near_sdk::serde")]
pub struct TierConfig {
    /// Minimum stake amount to get tier.
    pub min_point: Balance,
    /// Pool Weight for each tier.
    pub pool_weight: u32,
    /// Tracks number of participants for each tier.
    pub number_of_participants: u32,
}

impl TierConfig {
    pub fn new(min_point: Balance, pool_weight: u32, number_of_participants: u32) -> Self {
        Self {
            min_point,
            pool_weight,
            number_of_participants,
        }
    }

    pub fn get_default_tier_configs() -> TierConfigsType{
        let mut config = TierConfigsType::new();
        let e18 = u128::pow(10, 18);
        
        config.insert(Tier::Tier0, TierConfig::new(0, 0, 0));
        config.insert(Tier::Tier1, TierConfig::new(2_000 * e18, 1, 0));
        config.insert(Tier::Tier2, TierConfig::new(4_000 * e18, 2, 0));
        config.insert(Tier::Tier3, TierConfig::new(10_000 * e18, 5, 0));
        config.insert(Tier::Tier4, TierConfig::new(17_500 * e18, 9, 0));
        config.insert(Tier::Tier5, TierConfig::new(35_000 * e18, 16, 0));
        config.insert(Tier::Tier6, TierConfig::new(90_000 * e18, 32, 0));
        config.insert(Tier::Tier7, TierConfig::new(175_000 * e18, 52, 0));

        config
    }
}

impl Contract {
    pub(crate) fn internal_get_tier(&self, point: Balance) -> Tier { 
        let mut configs = self.tier_configs.iter().map(|a| (*a.0, *a.1)).collect::<Vec<(Tier, TierConfig)>>();
        // Sort the list descending by min point
        configs.sort_by(|a, b| b.1.min_point.cmp(&a.1.min_point));

        for (tier, config) in configs {
            if point >= config.min_point {
                return tier;
            }
        }
        Tier::Tier0
    }
}