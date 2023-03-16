#![deny(warnings)]
use std::sync::{Arc, RwLock};

use axum::{routing, Router};
use recipers::repository::Repository;

use tower_http::trace::TraceLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use crate::handler::{
    recipe_delete, recipe_get, recipe_put, recipe_share, recipes_get, recipes_post,
};

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
    let app = router(repository.clone());
    tracing::debug!("listening to 0.0.0.0:8080");
    axum::Server::bind(&"0.0.0.0:8080".parse().unwrap())
        .serve(app.into_make_service())
        .await?;

    Ok(())
}

fn router(state: AppState) -> Router {
    Router::new()
        .route("/", routing::get(|| async { "Hello World!" }))
        .route(
            "/cookbook/recipe",
            routing::get(recipes_get)
                .post(recipes_post)
                .with_state(state.clone())
                .layer(TraceLayer::new_for_http()),
        )
        .route(
            "/cookbook/recipe/:id",
            routing::get(recipe_get)
                .put(recipe_put)
                .delete(recipe_delete)
                .with_state(state.clone())
                .layer(TraceLayer::new_for_http()),
        )
        .route(
            "/cookbook/recipe/share",
            routing::get(recipe_share).with_state(state.clone()),
        )
}

type AppState = Arc<RwLock<Repository>>;

mod handler;

#[cfg(test)]
mod test {

    use std::error::Error;
    use std::ops::Bound;
    use std::sync::{Arc, RwLock};

    use http::Method;
    use hyper::StatusCode;
    use hyper::{body::to_bytes, Body, Request};
    use recipers::{Recipe, TableOfContents};

    use recipers::repository::Repository;
    use uuid::Uuid;

    use crate::router;

    use tower::Service;
    use tower::ServiceExt;

    mod fixture;

    type TestResult = Result<(), Box<dyn Error>>;

    #[tokio::test]
    async fn get_toc_empty() -> TestResult {
        let request = Request::builder()
            .uri("/cookbook/recipe")
            .header("Range", "bytes=0-9")
            .body(Body::empty())
            .unwrap();

        let repository = Arc::new(RwLock::new(Repository::new()));
        let mut app = router(repository.clone());
        let service = app.ready().await?;
        let response = service.call(request).await?;

        assert_eq!(response.status(), StatusCode::OK);

        // Der Response Body darf nur einmal gelesen werden, sonst
        // gibt es einen Fehler. Die Funktion into_body() konsumiert
        // das Response-Objekt, weshalb die Auswertung des Body zum
        // Schluss erfolgt. Das Response-Objekt ist danach nicht mehr
        // benutzbar.
        let body = to_bytes(response.into_body()).await?;
        let toc: TableOfContents = serde_json::from_slice(&body)?;

        assert_eq!(toc, TableOfContents::empty());

        Ok(())
    }

    #[tokio::test]
    async fn get_toc_filled() -> TestResult {
        // given
        let repository = Arc::new(RwLock::new(Repository::new()));
        let all_recipes = fixture::all_recipes()?;

        let ids = repository
            .write()
            .as_mut()
            .map(|r| r.insert_all(&all_recipes))
            .unwrap()?;

        let want = TableOfContents {
            total: all_recipes.len() as u64,
            content: ids
                .iter()
                .zip(all_recipes.iter())
                .map(|pair| pair.into())
                .collect(),
        };

        let pair = (&want, &vec!["cookbook", "recipe"]);
        let want = crate::handler::TableOfContents::from(&pair);

        // when

        let request = Request::builder()
            .uri("/cookbook/recipe")
            .header("Range", "bytes=0-9")
            .body(Body::empty())
            .unwrap();

        let mut app = router(repository.clone());
        let service = app.ready().await?;
        let response = service.call(request).await?;

        assert_eq!(response.status(), StatusCode::OK);

        let body = to_bytes(response.into_body()).await?;
        let got: crate::handler::TableOfContents = serde_json::from_slice(&body)?;

        // then
        assert_eq!(got, want);

        Ok(())
    }

    #[tokio::test]
    async fn get_recipe() -> TestResult {
        let repository = Arc::new(RwLock::new(Repository::new()));
        let all_recipes = fixture::all_recipes()?;

        {
            let mut w = repository.write().unwrap();
            w.insert_all(&all_recipes)?;
        }

        // when

        let read_lock = repository.read().unwrap();
        let all = (Bound::Unbounded, Bound::Unbounded);
        let toc = read_lock.list(&all, "Lasagne")?;
        let id = toc.content[0].id;

        let request = Request::builder()
            .uri(format!("/cookbook/recipe/{}", id))
            .header("Range", "bytes=0-9")
            .body(Body::empty())
            .unwrap();

        let mut app = router(repository.clone());
        let service = app.ready().await?;
        let response = service.call(request).await?;

        assert_eq!(response.status(), StatusCode::OK);

        let body = to_bytes(response.into_body()).await?;
        let got: Recipe = serde_json::from_slice(&body)?;
        let want = all_recipes.iter().find(|r| r.title == "Lasagne").unwrap();

        assert_eq!(&got, want);

        Ok(())
    }

    #[tokio::test]
    async fn update_new_recipe() -> TestResult {
        let repository = Arc::new(RwLock::new(Repository::new()));
        let id = uuid::Uuid::new_v4();
        let request = Request::builder()
            .method(Method::PUT)
            .uri(format!("/cookbook/recipe/{}", id))
            .header("Content-Type", "application/json")
            .body(fixture::LASAGNE.into())?;

        let mut app = router(repository.clone());
        let service = app.ready().await?;
        let response = service.call(request).await?;

        assert_eq!(response.status(), StatusCode::CREATED);

        let r = repository.read().unwrap();
        let got = r.get(&id)?;

        assert!(got.is_some());

        Ok(())
    }

    #[tokio::test]
    async fn replace_existing_recipe() -> TestResult {
        let repository = Arc::new(RwLock::new(Repository::new()));
        let lasagne: Recipe = fixture::LASAGNE.parse()?;

        let id = {
            let mut w = repository.write().unwrap();
            w.insert(&lasagne)?
        };

        let request = Request::builder()
            .method(Method::PUT)
            .uri(format!("/cookbook/recipe/{}", id))
            .header("Content-Type", "application/json")
            .body(fixture::LASAGNE.into())?;

        let mut app = router(repository.clone());
        let service = app.ready().await?;
        let response = service.call(request).await?;

        assert_eq!(response.status(), StatusCode::OK);
        Ok(())
    }

    #[tokio::test]
    async fn delete_non_existing_recipe() -> TestResult {
        let repository = Arc::new(RwLock::new(Repository::new()));

        let id = Uuid::new_v4();

        let request = Request::builder()
            .method(Method::DELETE)
            .uri(format!("/cookbook/recipe/{}", id))
            .body(Body::empty())?;

        let mut app = router(repository.clone());
        let service = app.ready().await?;
        let response = service.call(request).await?;

        assert_eq!(response.status(), StatusCode::NO_CONTENT);

        Ok(())
    }

    #[tokio::test]
    async fn delete_exiting_recipe() -> TestResult {
        let repository = Arc::new(RwLock::new(Repository::new()));

        let id = {
            let mut w = repository.write().unwrap();
            w.insert(&fixture::LASAGNE.parse()?)?
        };

        let request = Request::builder()
            .method(Method::DELETE)
            .uri(format!("/cookbook/recipe/{}", id))
            .body(Body::empty())?;

        let mut app = router(repository.clone());
        let service = app.ready().await?;
        let response = service.call(request).await?;

        assert_eq!(response.status(), StatusCode::NO_CONTENT);

        Ok(())
    }
}
