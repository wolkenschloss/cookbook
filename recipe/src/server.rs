use std::{
    ops::Bound,
    sync::{Arc, RwLock},
};

use axum::{
    extract::{Json, Path, Query, State, TypedHeader},
    headers::Range,
    http::{header, StatusCode},
    response::IntoResponse,
    routing, Router,
};
use recipers::{
    repository::{Repository, UpdateResult},
    Recipe,
};
use serde::Deserialize;
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
                .unwrap_or_else(|_| "server=debug,recipers=debug,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let repository = Arc::new(RwLock::new(Repository::new()));

    let app = Router::new()
        .route("/", routing::get(|| async { "Hello World!" }))
        .route(
            "/cookbook/recipe",
            routing::get(recipes_get)
                .post(recipes_post)
                .with_state(repository.clone())
                .layer(TraceLayer::new_for_http()),
        )
        .route(
            "/cookbook/recipe/:id",
            routing::get(recipe_get)
                .put(recipe_put)
                .delete(recipe_delete)
                .with_state(repository.clone())
                .layer(TraceLayer::new_for_http()),
        )
        .route(
            "/cookbook/recipe/share",
            routing::get(recipe_share).with_state(repository.clone()),
        );

    tracing::debug!("listening to 0.0.0.0:8080");
    axum::Server::bind(&"0.0.0.0:8080".parse().unwrap())
        .serve(app.into_make_service())
        .await?;

    Ok(())
}

type AppState = Arc<RwLock<Repository>>;

#[derive(Debug, Deserialize)]
struct Search {
    q: Option<String>,
}

async fn recipes_get(
    State(state): State<AppState>,
    Query(parameter): Query<Search>,
    TypedHeader(range): TypedHeader<Range>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let search = parameter.q.unwrap_or("".into());

    let it: (Bound<u64>, Bound<u64>) = range
        .iter()
        .nth(0)
        .unwrap_or((Bound::Unbounded, Bound::Unbounded));

    for r in range.iter() {
        tracing::debug!("found range {:?}", r)
    }

    let repository = state.read().unwrap();
    let toc = repository.list2(&it, &search).map_err(internal_error)?;

    Ok(Json(toc))
}

/// Utility function for mapping any error into a `500 Internal Server Error`
/// response.
fn internal_error<E>(err: E) -> (StatusCode, String)
where
    E: std::error::Error,
{
    (StatusCode::INTERNAL_SERVER_ERROR, err.to_string())
}

async fn recipes_post(
    State(state): State<AppState>,
    Json(payload): Json<Recipe>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    println!("recipes post called");
    println!("got recipe {:?}", payload);

    let mut repository = state.write().unwrap();
    let id = repository.insert(&payload).map_err(internal_error)?;

    Ok((
        StatusCode::CREATED,
        [(header::LOCATION, format!("/cookbook/recipe/{}", id))],
        Json(id),
    ))
}

async fn recipe_get(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let repository = state.read().map_err(internal_error)?;
    let recipe = repository.get(&id).map_err(internal_error)?;
    match recipe {
        Some(result) => Ok(Json(result.clone())),
        None => Err((StatusCode::NOT_FOUND, "recipe not found".to_owned())),
    }
}

async fn recipe_put(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(payload): Json<Recipe>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let mut repository = state.write().unwrap();
    let result = repository.update(&id, payload).map_err(internal_error)?;

    match result {
        UpdateResult::Created => Ok(StatusCode::OK.into_response()),
        UpdateResult::Changed => Ok((
            StatusCode::CREATED,
            [(header::LOCATION, format!("/cookbook/recipe/{}", id))],
            Json(id),
        )
            .into_response()),
    }
}

async fn recipe_delete(State(state): State<AppState>, Path(id): Path<Uuid>) {}
async fn recipe_share(State(state): State<AppState>) {}
