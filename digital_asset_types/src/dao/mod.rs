mod full_asset;
mod generated;
pub mod scopes;
pub use full_asset::*;
pub use generated::*;

use self::sea_orm_active_enums::{
    OwnerType, RoyaltyTargetType, SpecificationAssetClass, SpecificationVersions,
};
use sea_orm::{
    entity::*,
    sea_query::Expr,
    sea_query::{ConditionType, IntoCondition},
    Condition, DbErr, RelationDef,
};

pub struct GroupingSize {
    pub size: u64,
}

pub enum Pagination {
    Keyset {
        before: Option<Vec<u8>>,
        after: Option<Vec<u8>>,
    },
    Page {
        page: u64,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub struct SearchAssetsQuery {
    // Conditions
    pub negate: Option<bool>,
    /// Defaults to [ConditionType::All]
    pub condition_type: Option<ConditionType>,
    pub specification_version: Option<SpecificationVersions>,
    pub specification_asset_class: Option<SpecificationAssetClass>,
    pub owner_address: Option<Vec<u8>>,
    pub owner_type: Option<OwnerType>,
    pub creator_address: Option<Vec<u8>>,
    pub creator_verified: Option<bool>,
    pub authority_address: Option<Vec<u8>>,
    pub grouping: Option<(String, String)>,
    pub delegate: Option<Vec<u8>>,
    pub frozen: Option<bool>,
    pub supply: Option<u64>,
    pub supply_mint: Option<Vec<u8>>,
    pub compressed: Option<bool>,
    pub compressible: Option<bool>,
    pub royalty_target_type: Option<RoyaltyTargetType>,
    pub royalty_target: Option<Vec<u8>>,
    pub royalty_amount: Option<u32>,
    pub burnt: Option<bool>,
}

impl SearchAssetsQuery {
    pub fn count_conditions(&self) -> usize {
        // Initialize counter
        // todo ever heard of a flipping macro
        let mut num_conditions = 0;
        if self.specification_version.is_some() {
            num_conditions += 1;
        }
        if self.specification_asset_class.is_some() {
            num_conditions += 1;
        }
        if self.owner_address.is_some() {
            num_conditions += 1;
        }
        if self.owner_type.is_some() {
            num_conditions += 1;
        }
        if self.delegate.is_some() {
            num_conditions += 1;
        }
        if self.frozen.is_some() {
            num_conditions += 1;
        }
        if self.supply.is_some() {
            num_conditions += 1;
        }
        if self.supply_mint.is_some() {
            num_conditions += 1;
        }
        if self.compressed.is_some() {
            num_conditions += 1;
        }
        if self.compressible.is_some() {
            num_conditions += 1;
        }
        if self.royalty_target_type.is_some() {
            num_conditions += 1;
        }
        if self.royalty_target.is_some() {
            num_conditions += 1;
        }
        if self.royalty_amount.is_some() {
            num_conditions += 1;
        }
        if self.burnt.is_some() {
            num_conditions += 1;
        }
        if self.creator_address.is_some() {
            num_conditions += 1;
        }
        if self.creator_address.is_some() {
            num_conditions += 1;
        }
        if self.grouping.is_some() {
            num_conditions += 1;
        }

        num_conditions
    }

    pub fn conditions(&self) -> Result<(Condition, Vec<RelationDef>), DbErr> {
        let mut conditions = match self.condition_type {
            // None --> default to all when no option is provided
            None | Some(ConditionType::All) => Condition::all(),
            Some(ConditionType::Any) => Condition::any(),
        };

        let mut joins = Vec::new();

        conditions = conditions
            .add_option(
                self.specification_version
                    .clone()
                    .map(|x| asset::Column::SpecificationVersion.eq(x)),
            )
            .add_option(
                self.specification_asset_class
                    .clone()
                    .map(|x| asset::Column::SpecificationAssetClass.eq(x)),
            )
            .add_option(
                self.owner_address
                    .to_owned()
                    .map(|x| asset::Column::Owner.eq(x)),
            )
            .add_option(
                self.owner_type
                    .clone()
                    .map(|x| asset::Column::OwnerType.eq(x)),
            )
            .add_option(
                self.delegate
                    .to_owned()
                    .map(|x| asset::Column::Delegate.eq(x)),
            )
            .add_option(self.frozen.map(|x| asset::Column::Frozen.eq(x)))
            .add_option(self.supply.map(|x| asset::Column::Supply.eq(x)))
            .add_option(
                self.supply_mint
                    .to_owned()
                    .map(|x| asset::Column::SupplyMint.eq(x)),
            )
            .add_option(self.compressed.map(|x| asset::Column::Compressed.eq(x)))
            .add_option(self.compressible.map(|x| asset::Column::Compressible.eq(x)))
            .add_option(
                self.royalty_target_type
                    .clone()
                    .map(|x| asset::Column::RoyaltyTargetType.eq(x)),
            )
            .add_option(
                self.royalty_target
                    .to_owned()
                    .map(|x| asset::Column::RoyaltyTarget.eq(x)),
            )
            .add_option(
                self.royalty_amount
                    .map(|x| asset::Column::RoyaltyAmount.eq(x)),
            )
            .add_option(self.burnt.map(|x| asset::Column::Burnt.eq(x)));

        if let Some(c) = self.creator_address.to_owned() {
            let mut cond = Condition::all().add(asset_creators::Column::Creator.eq(c));
            if let Some(cv) = self.creator_verified {
                cond = cond.add(asset_creators::Column::Verified.eq(cv));
            }

            conditions = conditions.add(cond);

            let rel = asset_creators::Relation::Asset
                .def()
                .rev()
                .on_condition(|left, right| {
                    Expr::tbl(right, asset_creators::Column::AssetId)
                        .eq(Expr::tbl(left, asset::Column::Id))
                        .into_condition()
                });
            joins.push(rel);
        }

        if let Some(a) = self.authority_address.to_owned() {
            conditions = conditions.add(asset_authority::Column::Authority.eq(a.clone()));
            let rel = asset_authority::Relation::Asset
                .def()
                .rev()
                .on_condition(|left, right| {
                    Expr::tbl(right, asset_authority::Column::AssetId)
                        .eq(Expr::tbl(left, asset::Column::Id))
                        .into_condition()
                });
            joins.push(rel);
        }

        if let Some(g) = self.grouping.to_owned() {
            let cond = Condition::all()
                .add(asset_grouping::Column::GroupKey.eq(g.0))
                .add(asset_grouping::Column::GroupValue.eq(g.1));
            conditions = conditions.add(cond);
            let rel = asset_grouping::Relation::Asset
                .def()
                .rev()
                .on_condition(|left, right| {
                    Expr::tbl(right, asset_grouping::Column::AssetId)
                        .eq(Expr::tbl(left, asset::Column::Id))
                        .into_condition()
                });
            joins.push(rel);
        }

        Ok((
            match self.negate {
                None | Some(false) => conditions,
                Some(true) => conditions.not(),
            },
            joins,
        ))
    }
}
