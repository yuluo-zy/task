use serde::{Serialize, Deserialize};
use rocket::post;
use rocket::serde::json::{Json};

#[derive(Deserialize,Serialize)]
pub struct EmailTask {
    ct: String,
    to: String,
    subject: String,
    text: String,
    from: String,
    id: Option<String>,
    filePath: Option<String>,
    start: Option<String>,
    end: Option<String>,
    periodic: Option<String>,
    status: bool,
}

#[derive(Serialize, Deserialize)]
pub struct ResultTask<T> {
    code: i32,
    data: T,
    message: String,
}

impl<T> ResultTask<T> {
    pub fn success(data: T) -> Json<ResultTask<T>> {
        Json(ResultTask {
            code: 0,
            data,
            message: "".to_string(),
        })
    }

    pub fn error(data: T) -> Json<ResultTask<T>> {
        Json(ResultTask {
            code: 1,
            data,
            message: "".to_string(),
        })}
}

#[post("/message",format = "json", data = "<email_task>")]
pub fn add_email(email_task: Json<EmailTask>) -> Json<ResultTask<String>> {
    return ResultTask::<String>::success("success".to_string());
}

#[get("/messageCancel/<id>")]
pub fn delete_email(id: String) -> Json<ResultTask<String>> {
    return ResultTask::<String>::success(id);
}

#[post("/messageByPeriodic",format = "json", data = "<email_task>")]
pub fn send_periodic(email_task: Json<EmailTask>) -> Json<ResultTask<String>> {
    return ResultTask::<String>::success("success".to_string());
}

// use diesel::r2d2;
// use dotenv::dotenv;
// use fang::PgConnection;
// use fang::Queue;
// use fang::Queueable;
// use fang::RetentionMode;
// use fang::WorkerPool;
// use simple_worker::MyFailingTask;
// use simple_worker::MyTask;
// use std::env;
// use std::thread::sleep;
// use std::time::Duration;
//
// pub fn connection_pool(pool_size: u32) -> r2d2::Pool<r2d2::ConnectionManager<PgConnection>> {
//     let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
//
//     let manager = r2d2::ConnectionManager::<PgConnection>::new(database_url);
//
//     r2d2::Pool::builder()
//         .max_size(pool_size)
//         .build(manager)
//         .unwrap()
// }
//
// fn main()
//     dotenv().ok();
//
//     env_logger::init();
//
//     let queue = Queue::builder().connection_pool(connection_pool(3)).build();
//
//     let mut worker_pool = WorkerPool::<Queue>::builder()
//         .queue(queue)
//         .retention_mode(RetentionMode::KeepAll)
//         .number_of_workers(3_u32)
//         .task_type("worker_pool_test".to_string())
//         .build();
//
//     worker_pool.queue.insert_task(&MyTask::new(1)).unwrap();
//     worker_pool.queue.insert_task(&MyTask::new(1000)).unwrap();
//
//     worker_pool
//         .queue
//         .insert_task(&MyFailingTask::new(5000))
//         .unwrap();
//
//     worker_pool.start().unwrap();
//
//     sleep(Duration::from_secs(100))
// }
