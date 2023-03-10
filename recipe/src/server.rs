use std::sync::{Arc, RwLock};

use axum::{
    extract::{Json, Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing, Router,
};
use recipers::{repository::Repository, Recipe};
use uuid::Uuid;

use tower_http::trace::TraceLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
// use cookbook::recipe_service_server::{RecipeService, RecipeServiceServer};
// use cookbook::{ListTableOfContentsRequest, TableOfContentsResponse};

// use tonic::{transport::Server, Request, Response, Status};

// pub mod cookbook {
//     tonic::include_proto!("cookbook");
// }

// #[derive(Default)]
// pub struct MyService {}

// #[tonic::async_trait]
// impl RecipeService for MyService {
//     async fn list_table_of_contents(
//         &self,
//         request: Request<ListTableOfContentsRequest>,
//     ) -> Result<Response<TableOfContentsResponse>, Status> {
//         println!("Got a request from {:?}", request.remote_addr());
//         let reply = cookbook::TableOfContentsResponse {
//             greeting: "Das Wars".to_string(),
//         };

//         Ok(Response::new(reply))
//     }
// }

// #[tokio::main]
// async fn main() -> Result<(), Box<dyn std::error::Error>> {
//     let addr = "[::1]:50051".parse().unwrap();
//     let service = MyService::default();

//     println!("Service listening on {}", addr);
//     Server::builder()
//         .add_service(RecipeServiceServer::new(service))
//         .serve(addr)
//         .await?;

//     Ok(())
// }

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "example_todos=debug,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let repository = Arc::new(RwLock::new(Repository::new()));

    let app = Router::new()
        .route("/", routing::get(|| async { "Hello World!" }))
        .route(
            "/recipe",
            routing::get(recipes_get)
                .post(recipes_post)
                .with_state(repository.clone())
                .layer(TraceLayer::new_for_http()),
        )
        .route(
            "/recipe/:id",
            routing::get(recipe_get)
                .put(recipe_put)
                .delete(recipe_delete)
                .with_state(repository.clone()),
        )
        .route(
            "/recipe/share",
            routing::get(recipe_share).with_state(repository.clone()),
        );

    axum::Server::bind(&"0.0.0.0:8080".parse().unwrap())
        .serve(app.into_make_service())
        .await?;

    Ok(())
}

type AppState = Arc<RwLock<Repository>>;

use recipers::repository::Range;

async fn recipes_get(State(state): State<AppState>) {}
async fn recipes_post(
    State(state): State<AppState>,
    Json(payload): Json<Recipe>,
) -> impl IntoResponse {
    println!("recipes post called");
    println!("got recipe {:?}", payload);

    let mut repository = state.write().unwrap();
    match repository.insert(&payload) {
        Ok(id) => {
            match repository.list(&Range::Unbounded, "") {
                Ok(toc) => println!("repository contains {:?} elements", toc),
                Err(err) => todo!(),
            }

            (StatusCode::CREATED, Json(id)).into_response()
        }
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    }
}

async fn recipe_get(State(state): State<AppState>, Path(id): Path<Uuid>) {}
async fn recipe_put(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(payload): Json<Recipe>,
) {
}
async fn recipe_delete(State(state): State<AppState>, Path(id): Path<Uuid>) {}
async fn recipe_share(State(state): State<AppState>) {}
