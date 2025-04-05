#[cfg(feature = "old")]
use tosu_proxy::error::Result;
#[cfg(feature = "old")]
use tosu_proxy::old::run;

#[cfg(feature = "old")]
#[actix_web::main]
async fn main() -> Result<()> {
    run().await
}

#[cfg(feature = "new")]
fn main() {
    println!("no compile")
}