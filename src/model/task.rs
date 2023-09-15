use sqlx::FromRow;

use crate::{
    ctx::Ctx,
    model::{Error, Result},
};

use super::{
    base::{self, DbBmc},
    ModelManager,
};

#[derive(Debug, FromRow)]
pub struct Task {
    pub id: i64,
    pub title: String,
}

pub struct TaskForCreate {
    pub title: String,
}

pub struct TaskUpdate {
    pub title: Option<String>,
}

pub struct TaskBmc;

impl DbBmc for TaskBmc {
    const TABLE: &'static str = "task";
}

impl TaskBmc {
    pub async fn create(_ctx: &Ctx, mm: &ModelManager, task_c: TaskForCreate) -> Result<i64> {
        let db = mm.db();

        // query_as返回一个QueryAs的结构体，
        // 其中的O类型为实现了FromRow trait的类型 O: for<'r> FromRow<'r, DB::Row>,
        // 单独的i64是没有实现的，而元组的用宏实现了FromRow
        // implement FromRow for tuples of types that implement Decode
        // up to tuples of 9 values
        /* macro_rules! impl_from_row_for_tuple {
            ($( ($idx:tt) -> $T:ident );+;) => {
                impl<'r, R, $($T,)+> FromRow<'r, R> for ($($T,)+)
                where
                    R: Row,
                    usize: crate::column::ColumnIndex<R>,
                    $($T: crate::decode::Decode<'r, R::Database> + crate::types::Type<R::Database>,)+
                {
                    #[inline]
                    fn from_row(row: &'r R) -> Result<Self, Error> {
                        Ok(($(row.try_get($idx as usize)?,)+))
                    }
                }
            };
        }

        impl_from_row_for_tuple!(
            (0) -> T1;
        )
        */
        let (id,) = sqlx::query_as("INSERT INTO task (title) VALUES ($1) RETURNING id")
            .bind(task_c.title)
            .fetch_one(db)
            .await?; // 在这里可以使用？的原因是在model::Error中已经实现了From<sqlx::Error>

        Ok(id)
    }

    pub async fn list(_ctx: &Ctx, mm: &ModelManager) -> Result<Vec<Task>> {
        let db = mm.db();

        let tasks: Vec<Task> = sqlx::query_as("Select * from task order by id")
            .fetch_all(db)
            .await?;

        Ok(tasks)
    }

    pub async fn get(ctx: &Ctx, mm: &ModelManager, id: i64) -> Result<Task> {
        base::get::<Self, _>(ctx, mm, id).await
    }

    pub async fn delete(_ctx: &Ctx, mm: &ModelManager, id: i64) -> Result<()> {
        let db = mm.db();

        let count = sqlx::query("DELETE FROM task WHERE id = $1")
            .bind(id)
            .execute(db)
            .await?
            .rows_affected();

        if count == 0 {
            return Err(Error::EntityNotFound { entity: "task", id });
        }

        Ok(())
    }
}

// region:    ---Tests
#[cfg(test)]
mod tests {
    #![allow(unused)]
    use crate::_dev_utils;

    use super::*;
    use anyhow::Result;
    use serial_test::serial;

    #[serial]
    #[tokio::test]
    async fn test_create_ok() -> Result<()> {
        // Setup && Features
        let mm = _dev_utils::init_test().await;
        let ctx = Ctx::root_ctx();
        let fx_title = "test_create_ok Title";

        // --Exec
        let task_c = TaskForCreate {
            title: fx_title.to_string(),
        };
        let id = TaskBmc::create(&ctx, &mm, task_c).await?;

        // Check
        let task = TaskBmc::get(&ctx, &mm, id).await?;
        assert_eq!(task.title, fx_title);

        // --Cleanup
        TaskBmc::delete(&ctx, &mm, id).await?;

        Ok(())
    }

    #[serial]
    #[tokio::test]
    async fn test_get_error_not_found() -> Result<()> {
        // Setup && Features
        let mm = _dev_utils::init_test().await;
        let ctx = Ctx::root_ctx();
        let fx_id = 100;

        // --Exec
        let task = TaskBmc::get(&ctx, &mm, fx_id).await;
        assert!(
            matches!(
                task,
                Err(Error::EntityNotFound {
                    entity: "task",
                    id: fx_id
                })
            ),
            "EntityNotFound not matching"
        );

        Ok(())
    }

    #[serial]
    #[tokio::test]
    async fn test_list_ok() -> Result<()> {
        // Setup && Features
        let mm = _dev_utils::init_test().await;
        let ctx = Ctx::root_ctx();
        let fx_titles = &["test_list_ok 1", "test_list_ok 2"];
        _dev_utils::seed_tasks(&ctx, &mm, fx_titles).await?;

        // --Exec
        let tasks = TaskBmc::list(&ctx, &mm).await?;

        // Check
        let tasks = tasks
            .into_iter()
            .filter(|task| task.title.starts_with("test_list_ok"))
            .collect::<Vec<Task>>();
        assert_eq!(tasks.len(), 2, "number of seeded tasks.");

        // --Cleanup
        for task in tasks {
            TaskBmc::delete(&ctx, &mm, task.id).await?;
        }

        Ok(())
    }

    #[serial]
    #[tokio::test]
    async fn test_delete_error_not_found() -> Result<()> {
        // Setup && Features
        let mm = _dev_utils::init_test().await;
        let ctx = Ctx::root_ctx();
        let fx_id = 200;

        // --Exec
        let res = TaskBmc::delete(&ctx, &mm, fx_id).await;

        assert!(
            matches!(
                res,
                Err(Error::EntityNotFound {
                    entity: "task",
                    id: fx_id
                })
            ),
            "EntityNotFound not matching"
        );

        Ok(())
    }
}
// endregion: ---Tests
