use crate::fang_task_state::FangTaskState;
use crate::runnable::Runnable;
use crate::schema::fang_tasks;
use crate::CronError;
use crate::Scheduled::*;
use chrono::{DateTime, NaiveDateTime};
use chrono::Duration;
use chrono::Utc;
use cron::Schedule;
use diesel::prelude::*;
use diesel::r2d2;
use diesel::r2d2::ConnectionManager;
use diesel::r2d2::PoolError;
use diesel::r2d2::PooledConnection;
use diesel::result::Error as DieselError;
use sha2::Digest;
use sha2::Sha256;
use std::str::FromStr;
use thiserror::Error;
use typed_builder::TypedBuilder;
use uuid::Uuid;

#[cfg(test)]
use dotenv::dotenv;
#[cfg(test)]
use std::env;

pub type PoolConnection = PooledConnection<ConnectionManager<MysqlConnection>>;

#[derive(Queryable, Identifiable,AsChangeset, Debug, Eq, PartialEq, Clone, TypedBuilder)]
#[diesel(table_name = fang_tasks)]
pub struct Task {
    #[builder(setter(into))]
    pub id: String,
    #[builder(setter(into))]
    pub metadata: serde_json::Value,
    #[builder(setter(into))]
    pub error_message: Option<String>,
    #[builder(setter(into))]
    pub state: String,
    #[builder(setter(into))]
    pub task_type: String,
    #[builder(setter(into))]
    pub uniq_hash: Option<String>,
    #[builder(setter(into))]
    pub retries: i32,
    #[builder(setter(into))]
    pub scheduled_at: NaiveDateTime,
    #[builder(setter(into))]
    pub created_at: NaiveDateTime,
    #[builder(setter(into))]
    pub updated_at: NaiveDateTime,
}

#[derive(Insertable, Debug, Eq, PartialEq, Clone, TypedBuilder)]
#[diesel(table_name = fang_tasks)]
pub struct NewTask {
    #[builder(setter(into))]
    metadata: serde_json::Value,
    #[builder(setter(into))]
    task_type: String,
    #[builder(setter(into))]
    uniq_hash: Option<String>,
    #[builder(setter(into))]
    scheduled_at: NaiveDateTime,
}

#[derive(Debug, Error)]
pub enum QueueError {
    #[error(transparent)]
    DieselError(#[from] DieselError),
    #[error(transparent)]
    PoolError(#[from] PoolError),
    #[error(transparent)]
    CronError(#[from] CronError),
    #[error("Can not perform this operation if task is not uniq, please check its definition in impl Runnable")]
    TaskNotUniqError,
}

impl From<cron::error::Error> for QueueError {
    fn from(error: cron::error::Error) -> Self {
        QueueError::CronError(CronError::LibraryError(error))
    }
}

pub trait Queueable {
    fn fetch_and_touch_task(&self, task_type: String) -> Result<Option<Task>, QueueError>;

    fn insert_task(&self, params: &dyn Runnable) -> Result<Task, QueueError>;

    fn remove_all_tasks(&self) -> Result<usize, QueueError>;

    fn remove_all_scheduled_tasks(&self) -> Result<usize, QueueError>;

    fn remove_tasks_of_type(&self, task_type: &str) -> Result<usize, QueueError>;

    fn remove_task(&self, id: Uuid) -> Result<usize, QueueError>;

    /// To use this function task has to be uniq. uniq() has to return true.
    /// If task is not uniq this function will not do anything.
    fn remove_task_by_metadata(&self, task: &dyn Runnable) -> Result<usize, QueueError>;

    fn find_task_by_id(&self, id: Uuid) -> Option<Task>;

    fn update_task_state(&self, task: &Task, state: FangTaskState) -> Result<usize, QueueError>;

    fn fail_task(&self, task: &Task, error: &str) -> Result<usize, QueueError>;

    fn schedule_task(&self, task: &dyn Runnable) -> Result<Task, QueueError>;

    fn schedule_retry(
        &self,
        task: &Task,
        backoff_in_seconds: u32,
        error: &str,
    ) -> Result<Task, QueueError>;
}

#[derive(Clone, TypedBuilder)]
pub struct Queue {
    #[builder(setter(into))]
    pub connection_pool: r2d2::Pool<ConnectionManager<MysqlConnection>>,
}

impl Queueable for Queue {
    fn fetch_and_touch_task(&self, task_type: String) -> Result<Option<Task>, QueueError> {
        let mut connection = self.get_connection()?;

        Self::fetch_and_touch_query(&mut connection, task_type)
    }

    fn insert_task(&self, params: &dyn Runnable) -> Result<usize, QueueError> {
        let mut connection = self.get_connection()?;

        Self::insert_query(&mut connection, params, Utc::now().naive_utc())
    }
    fn schedule_task(&self, params: &dyn Runnable) -> Result<Task, QueueError> {
        let mut connection = self.get_connection()?;

        Self::schedule_task_query(&mut connection, params)
    }

    fn remove_all_scheduled_tasks(&self) -> Result<usize, QueueError> {
        let mut connection = self.get_connection()?;

        Self::remove_all_scheduled_tasks_query(&mut connection)
    }

    fn remove_all_tasks(&self) -> Result<usize, QueueError> {
        let mut connection = self.get_connection()?;

        Self::remove_all_tasks_query(&mut connection)
    }

    fn remove_tasks_of_type(&self, task_type: &str) -> Result<usize, QueueError> {
        let mut connection = self.get_connection()?;

        Self::remove_tasks_of_type_query(&mut connection, task_type)
    }

    fn remove_task(&self, id: Uuid) -> Result<usize, QueueError> {
        let mut connection = self.get_connection()?;

        Self::remove_task_query(&mut connection, id)
    }

    /// To use this function task has to be uniq. uniq() has to return true.
    /// If task is not uniq this function will not do anything.
    fn remove_task_by_metadata(&self, task: &dyn Runnable) -> Result<usize, QueueError> {
        if task.uniq() {
            let mut connection = self.get_connection()?;

            Self::remove_task_by_metadata_query(&mut connection, task)
        } else {
            Err(QueueError::TaskNotUniqError)
        }
    }

    fn update_task_state(&self, task: &Task, state: FangTaskState) -> Result<usize, QueueError> {
        let mut connection = self.get_connection()?;

        Self::update_task_state_query(&mut connection, task, state)
    }

    fn fail_task(&self, task: &Task, error: &str) -> Result<usize, QueueError> {
        let mut connection = self.get_connection()?;

        Self::fail_task_query(&mut connection, task, error)
    }

    fn find_task_by_id(&self, id: Uuid) -> Option<Task> {
        let mut connection = self.get_connection().unwrap();

        Self::find_task_by_id_query(&mut connection, id)
    }

    fn schedule_retry(
        &self,
        task: &Task,
        backoff_seconds: u32,
        error: &str,
    ) -> Result<Task, QueueError> {
        let mut connection = self.get_connection()?;

        Self::schedule_retry_query(&mut connection, task, backoff_seconds, error)
    }
}
fn get_naive_date_time(date_time :DateTime<Utc>) -> NaiveDateTime {
    date_time.naive_utc()
}
impl Queue {
    pub fn get_connection(&self) -> Result<PoolConnection, QueueError> {
        let result = self.connection_pool.get();

        if let Err(err) = result {
            log::error!("Failed to get a db connection {:?}", err);
            return Err(QueueError::PoolError(err));
        }

        Ok(result.unwrap())
    }

    pub fn schedule_task_query(
        connection: &mut MysqlConnection,
        params: &dyn Runnable,
    ) -> Result<Task, QueueError> {
        let scheduled_at = match params.cron() {
            Some(scheduled) => match scheduled {
                CronPattern(cron_pattern) => {
                    let schedule = Schedule::from_str(&cron_pattern)?;
                    let mut iterator = schedule.upcoming(Utc);

                    get_naive_date_time(iterator
                        .next()
                        .ok_or(QueueError::CronError(CronError::NoTimestampsError))?)
                }
                ScheduleOnce(datetime) => datetime,
            },
            None => {
                return Err(QueueError::CronError(CronError::TaskNotSchedulableError));
            }
        };

        Self::insert_query(connection, params, scheduled_at)
    }

    fn calculate_hash(json: String) -> String {
        let mut hasher = Sha256::new();
        hasher.update(json.as_bytes());
        let result = hasher.finalize();
        hex::encode(result)
    }

    pub fn insert_query(
        connection: &mut MysqlConnection,
        params: &dyn Runnable,
        scheduled_at: NaiveDateTime,
    ) -> Result<Task, QueueError> {
        if !params.uniq() {
            let new_task = NewTask::builder()
                .id(Uuid::new_v4().to_string())
                .scheduled_at(scheduled_at)
                .uniq_hash(None)
                .task_type(params.task_type())
                .metadata(serde_json::to_value(params).unwrap())
                .build();
            diesel::insert_into(fang_tasks::table)
                .values(new_task)
                .execute(connection)?;

            Ok(new_task)
        } else {
            let metadata = serde_json::to_value(params).unwrap();

            let uniq_hash = Self::calculate_hash(metadata.to_string());

            match Self::find_task_by_uniq_hash_query(connection, &uniq_hash) {
                Some(task) => Ok(task),
                None => {
                    let new_task = NewTask::builder()
                        .id(Uuid::new_v4().to_string())
                        .scheduled_at(scheduled_at)
                        .uniq_hash(Some(uniq_hash))
                        .task_type(params.task_type())
                        .metadata(serde_json::to_value(params).unwrap())
                        .build();
                    diesel::insert_into(fang_tasks::table)
                        .values(new_task)
                        .execute(connection)?;

                    Ok(new_task)
                }
            }
        }
    }

    pub fn fetch_task_query(connection: &mut MysqlConnection, task_type: String) -> Option<Task> {
        Self::fetch_task_of_type_query(connection, &task_type)
    }

    pub fn fetch_and_touch_query(
        connection: &mut MysqlConnection,
        task_type: String,
    ) -> Result<Option<Task>, QueueError> {
        connection.transaction::<Option<Task>, QueueError, _>(|conn| {
            let found_task = Self::fetch_task_query(conn, task_type);

            if found_task.is_none() {
                return Ok(None);
            }

            match Self::update_task_state_query(
                conn,
                &found_task.unwrap(),
                FangTaskState::InProgress,
            ) {
                Ok(updated_task) => Ok(Some(updated_task)),
                Err(err) => Err(err),
            }
        })
    }

    pub fn find_task_by_id_query(connection: &mut MysqlConnection, id: Uuid) -> Option<Task> {
        fang_tasks::table
            .filter(fang_tasks::id.eq(id.to_string()))
            .first::<Task>(connection)
            .ok()
    }

    pub fn remove_all_tasks_query(connection: &mut MysqlConnection) -> Result<usize, QueueError> {
        Ok(diesel::delete(fang_tasks::table).execute(connection)?)
    }

    pub fn remove_all_scheduled_tasks_query(
        connection: &mut MysqlConnection,
    ) -> Result<usize, QueueError> {
        let query = fang_tasks::table.filter(fang_tasks::scheduled_at.gt(Utc::now().naive_utc()));

        Ok(diesel::delete(query).execute(connection)?)
    }

    pub fn remove_tasks_of_type_query(
        connection: &mut MysqlConnection,
        task_type: &str,
    ) -> Result<usize, QueueError> {
        let query = fang_tasks::table.filter(fang_tasks::task_type.eq(task_type));

        Ok(diesel::delete(query).execute(connection)?)
    }

    pub fn remove_task_by_metadata_query(
        connection: &mut MysqlConnection,
        task: &dyn Runnable,
    ) -> Result<usize, QueueError> {
        let metadata = serde_json::to_value(task).unwrap();

        let uniq_hash = Self::calculate_hash(metadata.to_string());

        let query = fang_tasks::table.filter(fang_tasks::uniq_hash.eq(uniq_hash));

        Ok(diesel::delete(query).execute(connection)?)
    }

    pub fn remove_task_query(connection: &mut MysqlConnection, id: Uuid) -> Result<usize, QueueError> {
        let query = fang_tasks::table.filter(fang_tasks::id.eq(id.to_string()));

        Ok(diesel::delete(query).execute(connection)?)
    }

    pub fn update_task_state_query(
        connection: &mut MysqlConnection,
        task: &Task,
        state: FangTaskState,
    ) -> Result<Task, QueueError> {
        diesel::update(task)
            .set((
                fang_tasks::state.eq(state.into()),
                fang_tasks::updated_at.eq(Self::current_time().naive_utc()),
            ))
            .execute(connection)?;
        Ok(task.select(&task.id).first(connection))
    }

    pub fn fail_task_query(
        connection: &mut MysqlConnection,
        task: &Task,
        error: &str,
    ) -> Result<Task, QueueError> {
        diesel::update(task)
            .set((
                fang_tasks::state.eq(FangTaskState::Failed.into()),
                fang_tasks::error_message.eq(error),
                fang_tasks::updated_at.eq(Self::current_time().naive_utc()),
            ))
            .execute(connection)?;
        Ok(task.select(&task.id).first(connection))
    }

    fn current_time() -> DateTime<Utc> {
        Utc::now()
    }

    #[cfg(test)]
    pub fn connection_pool(pool_size: u32) -> r2d2::Pool<ConnectionManager<MysqlConnection>> {
        dotenv().ok();

        let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");

        let manager = ConnectionManager::<MysqlConnection>::new(database_url);

        r2d2::Pool::builder()
            .max_size(pool_size)
            .build(manager)
            .unwrap()
    }

    fn fetch_task_of_type_query(connection: &mut MysqlConnection, task_type: &str) -> Option<Task> {
        fang_tasks::table
            .order(fang_tasks::created_at.asc())
            .order(fang_tasks::scheduled_at.asc())
            .limit(1)
            .filter(fang_tasks::scheduled_at.le(Utc::now().naive_utc()))
            .filter(fang_tasks::state.eq_any(vec!["New", "Retried"]))
            .filter(fang_tasks::task_type.eq(task_type))
            .for_update()
            .skip_locked()
            .get_result::<Task>(connection)
            .ok()
    }

    fn find_task_by_uniq_hash_query(
        connection: &mut MysqlConnection,
        uniq_hash: &str,
    ) -> Option<Task> {
        fang_tasks::table
            .filter(fang_tasks::uniq_hash.eq(uniq_hash))
            .filter(fang_tasks::state.eq_any(vec!["New","Retried"]))
            .first::<Task>(connection)
            .ok()
    }

    pub fn schedule_retry_query(
        connection: &mut MysqlConnection,
        task: &Task,
        backoff_seconds: u32,
        error: &str,
    ) -> Result<Task, QueueError> {
        let now = Self::current_time();
        let scheduled_at = now + Duration::seconds(backoff_seconds as i64);

        let task = diesel::update(task)
            .set((
                fang_tasks::state.eq(FangTaskState::Retried.into()),
                fang_tasks::error_message.eq(error),
                fang_tasks::retries.eq(task.retries + 1),
                fang_tasks::scheduled_at.eq(scheduled_at.naive_utc()),
                fang_tasks::updated_at.eq(now.naive_utc()),
            ))
            .execute(connection)?;

        Ok(task)
    }
}
