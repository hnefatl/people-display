use rocket::*;

pub fn rocket() -> Rocket<Build> {
    rocket::build().mount("/", routes![index])
}

#[get("/")]
fn index() -> &'static str {
    "testing"
}
