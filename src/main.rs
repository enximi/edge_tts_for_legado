mod app;
mod config;
mod error;
mod http;
mod legado;
mod logging;
mod routes;
mod services;
mod startup;
mod state;
#[cfg(test)]
mod test_support;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    startup::run().await
}
