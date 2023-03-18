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

    use std::fmt::Debug;

    use std::sync::{Arc, RwLock, RwLockWriteGuard};

    use axum::response::Response;

    use axum::Router;
    use http::request::Builder;
    use http::Method;
    use hyper::{body::to_bytes, Request};
    use hyper::{Body, StatusCode};
    use recipers::{Recipe, TableOfContents};
    use serde::de::DeserializeOwned;

    use uuid::Uuid;

    use crate::{router, AppState};
    use recipers::repository::Repository;

    use tower::Service;
    use tower::ServiceExt;

    mod fixture;

    type TestResult = Result<(), Box<dyn Error>>;

    #[tokio::test]
    async fn get_toc_empty() -> TestResult {
        let mut testbed = Testbed::new();
        testbed
            .when(|r| {
                r.uri("/cookbook/recipe")
                    .header("Range", "bytes=0-9")
                    .body(Body::empty())
            })
            .await
            .status_eq(StatusCode::OK)
            .body_eq(&TableOfContents::empty())
            .await;

        // Der Response Body darf nur einmal gelesen werden, sonst
        // gibt es einen Fehler. Die Funktion into_body() konsumiert
        // das Response-Objekt, weshalb die Auswertung des Body zum
        // Schluss erfolgt. Das Response-Objekt ist danach nicht mehr
        // benutzbar.
        // let body = to_bytes(response.into_body()).await?;
        // let toc: TableOfContents = serde_json::from_slice(&body)?;

        // assert_eq!(toc, TableOfContents::empty());

        Ok(())
    }

    #[tokio::test]
    async fn get_toc_filled() -> TestResult {
        let mut testbed = Testbed::new();

        // given all recipes in repository
        let all_recipes = fixture::all_recipes()?;
        let ids = testbed.write().insert_all(&all_recipes)?;

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

        // when get table of contents
        testbed
            .when(|request| {
                request
                    .uri("/cookbook/recipe")
                    .header("Range", "bytes=0-9")
                    .body(Body::empty())
            })
            .await
            .status_eq(StatusCode::OK)
            .body_eq(&want)
            .await;

        Ok(())
    }

    #[tokio::test]
    async fn get_recipe() -> TestResult {
        let mut testbed = Testbed::new();
        let all_recipes = fixture::all_recipes()?;

        let ids = testbed.write().insert_all(&all_recipes)?;

        // when

        for id in ids {
            let uri = format!("/cookbook/recipe/{id}");

            let want = all_recipes.iter().find(|r| r.title == "Lasagne").unwrap();

            testbed
                .when(|r| r.uri(&uri).body(Body::empty()))
                .await
                .status_eq(StatusCode::OK)
                .body_eq(want)
                .await;
        }

        Ok(())
    }

    #[tokio::test]
    async fn update_new_recipe() -> TestResult {
        let mut testbed = Testbed::new();
        let id = uuid::Uuid::new_v4();
        let uri = format!("/cookbook/recipe/{id}");

        testbed
            .when(|r| {
                r.uri(&uri)
                    .method(Method::PUT)
                    .header("Content-Type", "application/json")
                    .body(fixture::LASAGNE.into())
            })
            .await
            .status_eq(StatusCode::CREATED);

        let res = testbed.read(&id);
        let recipe = res.unwrap();

        assert!(recipe.is_some());

        Ok(())
    }

    struct Testbed {
        state: AppState,
        _app: Router,
    }

    struct ResponseAssert {
        response: Response,
    }

    impl ResponseAssert {
        fn status_eq(self, want: StatusCode) -> Self {
            assert_eq!(self.response.status(), want);
            self
        }

        async fn body_eq<T: DeserializeOwned + PartialEq + Debug>(self, want: &T) {
            let got: T = self.extract_body::<T>().await;
            assert_eq!(got, *want);
        }

        #[allow(dead_code)]
        async fn body_ne<T: DeserializeOwned + PartialEq + Debug>(self, want: &T) {
            let got: T = self.extract_body::<T>().await;
            assert_ne!(got, *want);
        }

        async fn extract_body<T: DeserializeOwned + PartialEq + Debug>(self) -> T {
            let body = self.response.into_body();
            let bytes = to_bytes(body).await.unwrap();
            serde_json::from_slice(&bytes).unwrap()
        }
    }

    impl Testbed {
        fn new() -> Testbed {
            let state = Arc::new(RwLock::new(Repository::new()));
            Testbed {
                state: state.clone(),
                _app: router(state.clone()),
            }
        }

        fn write(&mut self) -> RwLockWriteGuard<Repository> {
            self.state.write().unwrap()
        }

        fn read<'a>(&'a self, id: &Uuid) -> Result<Option<Recipe>, Box<dyn Error + '_>> {
            let r = self.state.read()?;
            let opt = r.get(id)?;
            Ok(opt.cloned())
        }

        async fn when<F>(&mut self, f: F) -> ResponseAssert
        where
            F: FnOnce(Builder) -> Result<Request<Body>, http::Error>,
        {
            let service = self._app.ready().await.unwrap();
            let builder = Request::builder();
            let req = f(builder).unwrap();
            // Das ist ein Hack.

            let response = service.call(req).await.unwrap();
            ResponseAssert { response }
        }
    }

    #[tokio::test]
    async fn replace_existing_recipe() -> TestResult {
        // given
        let mut testbed = Testbed::new();

        let mut vegetarische_lasagne: Recipe = fixture::LASAGNE.parse()?;
        vegetarische_lasagne.title = "Vegetarische Lasagne".to_string();

        let id = testbed.write().insert(&vegetarische_lasagne)?;

        // when
        let uri = format!("/cookbook/recipe/{id}");
        testbed
            .when(|r| {
                r.uri(&uri)
                    .method(Method::PUT)
                    .header("Content-Type", "application/json")
                    .body(fixture::LASAGNE.into())
            })
            .await
            .status_eq(StatusCode::NO_CONTENT);

        let normale_lasagne = testbed.read(&id).unwrap().unwrap();
        assert_ne!(normale_lasagne, vegetarische_lasagne);

        Ok(())
    }

    #[tokio::test]
    async fn delete_non_existing_recipe() -> TestResult {
        let mut testbed = Testbed::new();

        let id = Uuid::new_v4();
        let uri = format!("/cookbook/recipe/{id}");

        testbed
            .when(|r| r.uri(&uri).method(Method::DELETE).body(Body::empty()))
            .await
            .status_eq(StatusCode::NO_CONTENT);

        Ok(())
    }

    #[tokio::test]
    async fn delete_exiting_recipe_refactored() -> TestResult {
        let mut testbed = Testbed::new();

        let id = testbed.write().insert(&fixture::LASAGNE.parse().unwrap())?;
        let uri = format!("/cookbook/recipe/{id}");

        testbed
            .when(|r| r.uri(&uri).method(Method::DELETE).body(Body::empty()))
            .await
            .status_eq(StatusCode::NO_CONTENT);

        Ok(())
    }

    #[tokio::test]
    async fn delete_exiting_recipe() -> TestResult {
        let mut testbed = Testbed::new();
        let id = testbed.write().insert(&fixture::LASAGNE.parse()?)?;

        testbed
            .when(|request| {
                request
                    .method(Method::DELETE)
                    .uri(format!("/cookbook/recipe/{}", id))
                    .body(Body::empty())
            })
            .await
            .status_eq(StatusCode::NO_CONTENT);

        Ok(())
    }
}
