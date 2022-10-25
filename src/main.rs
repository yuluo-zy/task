#[macro_use] extern crate rocket;
pub mod handler;
pub use handler::*;
#[launch]
fn rocket() -> _ {
    rocket::build().mount( "/email",routes![add_email, delete_email, send_periodic])
}