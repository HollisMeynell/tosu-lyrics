use crate::database::database;
use sea_orm::entity::prelude::*;
use sea_orm::sea_query::OnConflict;
use sea_orm::{ActiveValue, ColumnTrait, QueryFilter, QueryOrder};

// 注意: 不能 `use crate::error::Result` —— 会遮住 prelude 里 derive 宏依赖的
// `Result` 别名, 因此这里全部用 `crate::error::Result` 全限定书写。
type Res<T> = crate::error::Result<T>;

pub const SCOPE_BID: &str = "bid";
pub const SCOPE_SID: &str = "sid";
pub const SCOPE_TITLE: &str = "title";

pub fn is_valid_scope(scope: &str) -> bool {
    matches!(scope, SCOPE_BID | SCOPE_SID | SCOPE_TITLE)
}

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "lyric_block")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    #[sea_orm(indexed)]
    pub scope: String,
    /// 作用域的取值：bid / sid 存数字字符串，title 存标题本身
    #[sea_orm(indexed)]
    pub value: String,
    #[sea_orm(default_value = "")]
    pub title: String,
    /// 仅用于展示与旧 WS 协议兼容，不参与匹配
    #[sea_orm(default_value = 0)]
    pub sid: i32,
    #[sea_orm(default_value = "")]
    pub reason: String,
    #[sea_orm(default_value = 0)]
    pub created_at: i64,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}

impl Entity {
    pub async fn list_all() -> Res<Vec<Model>> {
        Ok(Self::find()
            .order_by_asc(Column::Id)
            .all(database())
            .await?)
    }

    /// 注意：不能叫 `find_by_id` —— 会盖住 `EntityTrait::find_by_id` 造成自递归
    pub async fn get_by_id(id: i32) -> Res<Option<Model>> {
        Ok(Self::find_by_id(id).one(database()).await?)
    }

    /// 幂等新增：同一 (scope, value) 已存在时只更新标题，不报错也不重复插入
    pub async fn upsert(
        scope: &str,
        value: &str,
        title: &str,
        sid: i32,
        reason: &str,
    ) -> Res<Model> {
        if !is_valid_scope(scope) {
            return Err(crate::error::Error::Runtime(format!(
                "未知的屏蔽作用域: {scope}"
            )));
        }
        if value.trim().is_empty() {
            return Err(crate::error::Error::Runtime(
                "屏蔽作用域取值不能为空".into(),
            ));
        }

        let now = sea_orm::sqlx::types::chrono::Utc::now().timestamp_millis();
        let model = ActiveModel {
            scope: ActiveValue::Set(scope.to_string()),
            value: ActiveValue::Set(value.to_string()),
            title: ActiveValue::Set(title.to_string()),
            sid: ActiveValue::Set(sid),
            reason: ActiveValue::Set(reason.to_string()),
            created_at: ActiveValue::Set(now),
            ..Default::default()
        };

        let mut on_conflict = OnConflict::columns([Column::Scope, Column::Value]);
        on_conflict
            .update_column(Column::Title)
            .update_column(Column::Sid)
            .update_column(Column::Reason);
        Self::insert(model)
            .on_conflict(on_conflict)
            .exec(database())
            .await?;

        Self::find()
            .filter(Column::Scope.eq(scope))
            .filter(Column::Value.eq(value))
            .one(database())
            .await?
            .ok_or(crate::error::Error::Impossible)
    }

    /// 不改作用域与取值：规则身份不变，不需要重算当前歌曲是否被屏蔽。
    pub async fn update_meta(id: i32, title: Option<&str>, reason: Option<&str>) -> Res<bool> {
        let mut update = Self::update_many();
        if let Some(title) = title {
            update = update.col_expr(
                Column::Title,
                sea_orm::sea_query::Expr::value(title.to_string()),
            );
        }
        if let Some(reason) = reason {
            update = update.col_expr(
                Column::Reason,
                sea_orm::sea_query::Expr::value(reason.to_string()),
            );
        }
        let result = update.filter(Column::Id.eq(id)).exec(database()).await?;
        Ok(result.rows_affected > 0)
    }

    /// 同上：避开 `EntityTrait::delete_by_id`
    pub async fn remove_by_id(id: i32) -> Res<bool> {
        let result = <Self as EntityTrait>::delete_by_id(id)
            .exec(database())
            .await?;
        Ok(result.rows_affected > 0)
    }

    /// 幂等删除，只影响这一个作用域
    pub async fn delete_by_scope_value(scope: &str, value: &str) -> Res<u64> {
        let result = Self::delete_many()
            .filter(Column::Scope.eq(scope))
            .filter(Column::Value.eq(value))
            .exec(database())
            .await?;
        Ok(result.rows_affected)
    }

    pub async fn delete_all() -> Res<u64> {
        let result = Self::delete_many().exec(database()).await?;
        Ok(result.rows_affected)
    }

    pub async fn count() -> Res<u64> {
        Ok(Self::find().count(database()).await?)
    }

    /// 判断某首歌是否被屏蔽。
    ///
    /// **不做跨作用域回退**：`bid` 命中就是命中，`title` 规则只在用户显式创建过
    /// 同名标题规则时才生效，不会因为"标题恰好相同"而命中别的谱面。
    pub async fn is_blocked(bid: i64, sid: i64, title: &str) -> Res<Option<Model>> {
        let all = Self::list_all().await?;
        let bid_s = bid.to_string();
        let sid_s = sid.to_string();

        // 优先级：bid > sid > title（仅用于返回哪条规则命中，不改变命中判定）
        if let Some(m) = all
            .iter()
            .find(|m| m.scope == SCOPE_BID && m.value == bid_s)
        {
            return Ok(Some(m.clone()));
        }
        if let Some(m) = all
            .iter()
            .find(|m| m.scope == SCOPE_SID && m.value == sid_s)
        {
            return Ok(Some(m.clone()));
        }
        let title_hit = if title.is_empty() {
            None
        } else {
            all.iter()
                .find(|m| m.scope == SCOPE_TITLE && m.value == title)
        };
        if let Some(m) = title_hit {
            return Ok(Some(m.clone()));
        }
        Ok(None)
    }
}
