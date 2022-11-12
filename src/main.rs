use axum::{
    async_trait,
    body::{Bytes, HttpBody},
    extract::{rejection::BytesRejection, FromRequest, RequestParts},
    http::StatusCode,
    response::IntoResponse,
    routing::get,
    BoxError, Router,
};
use mongodb::{bson::doc, options::ListDatabasesOptions, Client, Database};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use validator::Validate;

mod custom_response;

#[tokio::main]
async fn main() {
    let app_settings = AppSettings::new();

    let app_state = initialize_infrastructure(&app_settings).await;

    let app = Router::new().route("/hello", get(hello_handler));

    println!("Server started at port: {}", app_settings.server.port);

    axum::Server::bind(
        &format!("0.0.0.0:{}", app_settings.server.port)
            .parse()
            .unwrap(),
    )
    .serve(app.into_make_service())
    .await
    .unwrap();
}

async fn hello_handler() -> &'static str {
    "New update! \n"
}

async fn initialize_infrastructure(app_settings: &AppSettings) -> AppState {
    let db = match connect_to_the_mongo(&app_settings.db).await {
        Ok(db) => db,
        Err(err) => {
            println!("{}", err);
            panic!("Unable to connect to mongo db!");
        }
    };

    AppState { db }
}

async fn connect_to_the_mongo(
    mongo_config: &MongoConfig,
) -> Result<Database, mongodb::error::Error> {
    //let client = Client::with_uri_str("mongodb+srv://dca:8DnryMrPQfZNa0CZ@cluster0.rhehpau.mongodb.net/?retryWrites=true&w=majority").await?;
    let client = Client::with_uri_str(mongo_config.uri).await?;
    let db = client.database(mongo_config.db_name);

    // Ping the server to see if you can connect to the cluster
    db.run_command(doc! {"ping": 1}, None).await?;

    println!("Successfully connected to the database!");

    Ok(db)
}

struct AppState {
    db: Database,
}

struct AppSettings {
    server: Server,
    db: MongoConfig,
}

struct MongoConfig {
    uri: &'static str,
    db_name: &'static str,
}

struct Server {
    port: &'static str,
}

impl AppSettings {
    fn new() -> Self {
        Self {
            server: Server { port: "8080" },
            db: MongoConfig {
                uri: "mongodb://localhost:27017",
                db_name: "dca",
            },
        }
    }
}

#[derive(Debug, Serialize)]
struct Paginated<T> {
    items: Vec<T>,
    page: u32,
    size: u32,
    pages_count: u32,
    total_count: u32,
}

pub struct Validated<T>(pub T);

#[async_trait]
impl<B, T> FromRequest<B> for Validated<T>
where
    B: HttpBody + Send,
    B::Data: Send,
    B::Error: Into<BoxError>,
    T: DeserializeOwned + Validate,
{
    type Rejection = ValidateRejection;

    async fn from_request(req: &mut RequestParts<B>) -> Result<Self, Self::Rejection> {
        let bytes = Bytes::from_request(req).await?;
        let value = serde_json::from_slice::<T>(&bytes)?;
        value.validate()?;

        Ok(Validated(value))
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ValidateRejection {
    #[error(transparent)]
    BytesRejection(#[from] BytesRejection),
    #[error(transparent)]
    DeserializeRejection(#[from] serde_json::error::Error),
    #[error(transparent)]
    ValidationError(#[from] validator::ValidationErrors),
}

impl IntoResponse for ValidateRejection {
    fn into_response(self) -> axum::response::Response {
        match self {
            Self::BytesRejection(e) => {
                println!("[VALIDATE Bytes Rejection error: {}", e.to_string())
            }
            Self::DeserializeRejection(e) => {
                println!("[VALIDATE Deserialize Rejection error: {}", e.to_string())
            }
            Self::ValidationError(e) => {
                let field_errors = e.field_errors();
                println!("{:?}", field_errors)
            }
        }

        (StatusCode::BAD_REQUEST, "Invalid input data!").into_response()
    }
}
