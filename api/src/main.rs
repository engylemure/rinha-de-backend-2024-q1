#[global_allocator]
static ALLOC: snmalloc_rs::SnMalloc = snmalloc_rs::SnMalloc;

#[actix_web::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    server::server().await
}
