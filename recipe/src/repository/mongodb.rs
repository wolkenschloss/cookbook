use std::{cmp::min, collections::HashMap, error::Error, ops::Bound, primitive};

use bson::Regex;
use mongodb::{
    bson::{self, doc},
    options::{
        CountOptions, EstimatedDocumentCountOptions, FindOneAndReplaceOptions, FindOptions,
        ReplaceOptions, SelectionCriteria, ServerApi,
    },
    sync::{Client, Collection, Database},
};
use uuid::Uuid;

use crate::{repository::BoundExt, Recipe, Summary, TableOfContents};

use super::{RepositoryError, UpdateResult};

use serde::{self, Deserialize, Serialize};

#[derive(Clone)]
pub struct MongoDbClient {
    _database: Database,
    pub collection: Collection<Entity<Recipe>>,
}

fn to_repository_error(err: mongodb::error::Error) -> RepositoryError {
    RepositoryError::MongoDb
}
impl MongoDbClient {
    pub fn new() -> Result<MongoDbClient, Box<dyn Error>> {
        let uri = "mongodb://127.0.0.1:27017";

        let client = Client::with_uri_str(uri)?;
        let database = client.database("cookbook");
        let collection = database.collection::<Entity<Recipe>>("recipes");

        tracing::info!("creating mongodb client for {uri}");
        Ok(MongoDbClient {
            _database: database,
            collection,
        })
    }
}

impl Drop for MongoDbClient {
    fn drop(&mut self) {
        tracing::info!("closing mongodb client")
    }
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug)]
pub struct Entity<T> {
    _id: bson::Uuid,
    #[serde(flatten)]
    data: T,
}

impl super::Repository for MongoDbClient {
    fn insert(&mut self, r: &crate::Recipe) -> Result<uuid::Uuid, super::RepositoryError> {
        let entity = Entity {
            _id: Uuid::new_v4().into(),
            data: r.clone(),
        };

        match self.collection.insert_one(entity.clone(), None) {
            Ok(insert) => Ok(entity._id.into()),
            Err(whatever) => Err(RepositoryError::Poison),
        }
    }

    fn insert_all(
        &mut self,
        recipes: &[crate::Recipe],
    ) -> Result<Vec<uuid::Uuid>, super::RepositoryError> {
        let entities = recipes
            .iter()
            .map(|r| Entity {
                _id: Uuid::new_v4().into(),
                data: r.clone(),
            })
            .collect::<Vec<Entity<Recipe>>>();

        match self.collection.insert_many(entities.iter(), None) {
            Err(err) => {
                tracing::error!("error inserting recipes: {:?}", err);
                println!("error inserting recipes: {:?}", err);
                Err(RepositoryError::Poison)
            }
            Ok(ok) => {
                tracing::debug!("mongodb client: insert_all => {:?}", ok);
                Ok(entities.iter().map(|e| e._id.into()).collect())
            }
        }
    }

    fn list(
        &self,
        range: &(std::ops::Bound<u64>, std::ops::Bound<u64>),
        search: &str,
    ) -> Result<crate::TableOfContents, super::RepositoryError> {
        let filter1 = doc! {"title": Regex {pattern: search.to_owned(), options: "".to_owned()}};
        let filter2 = doc! {"title": Regex {pattern: search.to_owned(), options: "".to_owned()}};
        let projection = doc! {"_id": 1, "title": 1};
        let sort = doc! {"title": 1};

        let total = self
            .collection
            .count_documents(
                doc! {"title": Regex {pattern: search.to_owned(), options: "".to_owned()}},
                None,
            )
            .map_err(to_repository_error)?;

        println!("---->>>>  Anzahl der Elemente ist {total}");

        // normalize

        // let total = self
        //     .collection
        //     .estimated_document_count(EstimatedDocumentCountOptions::builder()
        //         .
        //     )
        //     .count_documents(filter1, None)
        //     .map_err(to_repository_error)?;

        let (skip, limit) = match range.0 {
            Bound::Unbounded => match range.1 {
                Bound::Unbounded => (0, total),
                Bound::Excluded(e) => (0, min(total, e - 1)),
                Bound::Included(i) => (0, min(total, i + 1)),
            },
            Bound::Included(i) => match range.1 {
                Bound::Unbounded => (i, total - i),
                Bound::Excluded(e) => (i, min(total, e) - i),
                Bound::Included(n) => (i, min(total, n + 1) - i),
            },
            Bound::Excluded(x) => match range.1 {
                Bound::Unbounded => (x + 1, total - x + 1),
                Bound::Excluded(x2) => (x + 1, min(total, x2 + 1) - x + 1),
                Bound::Included(i) => (x + 1, min(total, i) - x + 1),
            },
        };

        println!("list: skip {skip}, limit {limit}");

        // Wenn für limit der Wert 0 übergeben wird, ist das identisch zu
        // unbegrenztem limit. Deshalb wird an dieser Stelle abgebrochen
        // und ein leeres Inhaltsverzeichnis zurückgegeben.

        if limit == 0 {
            return Ok(TableOfContents {
                total: total,
                content: vec![],
            });
        }

        // let (skip, limit, total) = (0, 100, 100);
        let options = FindOptions::builder()
            .skip(skip)
            .limit(Some(limit as i64))
            .sort(sort)
            .build();

        let mut cursor = self
            .collection
            // .find(doc! {}, None)
            .find(filter2, Some(options))
            .map_err(to_repository_error)?;

        // while let Some(result) = cursor.next() {
        //     if let Ok(entity) = result {
        //         println!("Entoty {:?}", entity)
        //     };
        // }

        let content: Vec<_> = cursor
            .into_iter()
            .filter_map(|f| f.ok())
            .map(|item| Summary {
                id: item._id.into(),
                title: item.data.title,
            })
            .collect();

        Ok(TableOfContents { total, content })
    }

    fn get(&self, id: &uuid::Uuid) -> Result<Option<crate::Recipe>, super::RepositoryError> {
        let filter = doc! {"_id": id};
        let options = FindOptions::builder().batch_size(1).build();
        let mut cursor = self.collection.find(filter, options).unwrap();
        while let Some(entity) = cursor.next() {
            tracing::debug!("found entity {:?}", entity);
            if let Ok(x) = entity {
                let res = Some(Recipe {
                    title: x.data.title,
                    preparation: x.data.preparation,
                    servings: x.data.servings,
                    ingredients: x.data.ingredients,
                });

                return Ok(res);
            }
        }

        Ok(None)
    }

    fn remove(&mut self, id: &uuid::Uuid) -> Result<(), super::RepositoryError> {
        let result = self
            .collection
            .delete_one(doc! {"_id": id}, None)
            .map_err(to_repository_error)?;

        let count = result.deleted_count;
        tracing::debug!("recipe deleted. id = {id}, count = {count}");
        Ok(())
    }

    fn update(
        &mut self,
        id: &uuid::Uuid,
        recipe: &crate::Recipe,
    ) -> Result<super::UpdateResult, super::RepositoryError> {
        let entity = Entity {
            _id: (*id).into(),
            data: recipe.clone(),
        };

        let options = ReplaceOptions::builder().upsert(true).build();

        let result = self
            .collection
            .replace_one(doc! {"_id": id}, entity, Some(options))
            .map_err(to_repository_error)?;

        if let Some(_x) = result.upserted_id {
            return Ok(UpdateResult::Created);
        }

        Ok(UpdateResult::Changed)
    }
}
