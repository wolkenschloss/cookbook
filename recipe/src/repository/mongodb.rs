use std::{cmp::min, collections::HashMap, ops::Bound};

use uuid::Uuid;

use crate::{repository::BoundExt, Recipe, Summary, TableOfContents};

use super::{RepositoryError, UpdateResult};

pub struct MongoDbClient;

impl MongoDbClient {
    pub fn new() -> MongoDbClient {
        MongoDbClient {}
    }
}

impl super::Repository for MongoDbClient {
    fn insert(&mut self, r: &crate::Recipe) -> Result<uuid::Uuid, super::RepositoryError> {
        todo!()
    }

    fn insert_all(
        &mut self,
        recipes: &[crate::Recipe],
    ) -> Result<Vec<uuid::Uuid>, super::RepositoryError> {
        todo!()
    }

    fn list(
        &self,
        range: &(std::ops::Bound<u64>, std::ops::Bound<u64>),
        search: &str,
    ) -> Result<crate::TableOfContents, super::RepositoryError> {
        todo!()
    }

    fn get(&self, id: &uuid::Uuid) -> Result<Option<&crate::Recipe>, super::RepositoryError> {
        todo!()
    }

    fn remove(&mut self, id: &uuid::Uuid) -> Result<(), super::RepositoryError> {
        todo!()
    }

    fn update(
        &mut self,
        id: &uuid::Uuid,
        recipe: crate::Recipe,
    ) -> Result<super::UpdateResult, super::RepositoryError> {
        todo!()
    }
}
