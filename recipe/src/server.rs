use std::sync::{Arc, RwLock};

use axum::{routing, Router};
use recipers::repository::Repository;

use tower_http::trace::TraceLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use crate::handler::{
    recipe_delete, recipe_get, recipe_put, recipe_share, recipes_get, recipes_post,
};
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

mod handler {
    use std::ops::Bound;

    use axum::{
        extract::{Path, Query, State},
        headers::Range,
        http::{header, StatusCode},
        response::IntoResponse,
        Json, TypedHeader,
    };
    use recipers::{repository::UpdateResult, Recipe};
    use serde::{Deserialize, Serialize};
    use uuid::Uuid;

    use crate::AppState;

    #[derive(Debug, Deserialize)]
    pub struct Search {
        q: Option<String>,
    }

    #[derive(Serialize)]
    struct TableOfContents {
        total: u64,
        content: Vec<Content>,
    }

    impl From<&(&recipers::TableOfContents, &Vec<&str>)> for TableOfContents {
        fn from(value: &(&recipers::TableOfContents, &Vec<&str>)) -> Self {
            let toc = value.0;
            let path = value.1;

            let x = toc
                .content
                .iter()
                .map(|item| Content {
                    title: item.title.clone(),
                    recipe_id: item.id,
                    links: Links::new(item.id, path),
                })
                .collect();

            TableOfContents {
                total: toc.total,
                content: x,
            }
        }
    }

    #[derive(Serialize)]
    struct Content {
        title: String,
        #[serde(rename = "recipeId")]
        recipe_id: Uuid,
        #[serde(rename = "_links")]
        links: Links,
    }

    #[derive(Serialize)]
    struct Links {
        #[serde(rename = "self")]
        myself: Link,
    }

    impl Links {
        fn new(id: Uuid, path: &[&str]) -> Links {
            let href = format!(
                "http://localhost:8181/{}",
                [path.join("/"), id.to_string()].join("/")
            );

            Links {
                myself: Link { href },
            }
        }
    }

    #[derive(Serialize)]
    struct Link {
        href: String,
    }

    pub async fn recipes_get(
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
        let path = &vec!["cookbook", "recipe"];
        let pair = (&toc, path);
        Ok(Json(TableOfContents::from(&pair)))
    }

    /// Utility function for mapping any error into a `500 Internal Server Error`
    /// response.
    fn internal_error<E>(err: E) -> (StatusCode, String)
    where
        E: std::error::Error,
    {
        (StatusCode::INTERNAL_SERVER_ERROR, err.to_string())
    }

    pub async fn recipes_post(
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

    pub async fn recipe_get(
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

    pub async fn recipe_put(
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

    pub async fn recipe_delete(State(_state): State<AppState>, Path(_id): Path<Uuid>) {}
    pub async fn recipe_share(State(_state): State<AppState>) {}
}
