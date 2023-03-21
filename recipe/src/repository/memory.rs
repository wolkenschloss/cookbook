use std::{cmp::min, collections::HashMap, ops::Bound};

use uuid::Uuid;

use crate::{repository::BoundExt, Recipe, Summary, TableOfContents};

use super::{RepositoryError, UpdateResult};

/// An in-memory repository for recipes
pub struct Ephemeral {
    entries: HashMap<Uuid, Recipe>,
}

impl Ephemeral {
    /// Creates a new repository
    pub fn new() -> Ephemeral {
        Ephemeral {
            entries: HashMap::new(),
        }
    }
}

impl super::Repository for Ephemeral {
    /// Adds a recipe to the repository
    fn insert(&mut self, r: &Recipe) -> Result<Uuid, RepositoryError> {
        let id = Uuid::new_v4();
        self.entries.insert(id, r.clone());
        Ok(id)
    }

    /// Adds all recipes to the repository and returns
    /// a Vector of Idents.
    fn insert_all(&mut self, recipes: &[Recipe]) -> Result<Vec<Uuid>, RepositoryError> {
        recipes.iter().map(|r| self.insert(&r)).collect()
    }

    /// Creates a table of contents for the specified filter
    /// criteria.
    ///
    /// The recipes are sorted by name. All recipes that start with
    /// "search" are included in the table of contents. The table of
    /// contents contains all the recipes within the given range.
    fn list(
        &self,
        range: &(Bound<u64>, Bound<u64>),
        search: &str,
    ) -> Result<TableOfContents, RepositoryError> {
        let mut summaries: Vec<Summary> = self
            .entries
            .iter()
            .map(|entity| entity.into())
            .filter(|s: &Summary| s.title.starts_with(search))
            .collect();

        summaries.sort();

        tracing::debug!("Got range {:?}", range);

        let xrange = if summaries.len() == 0 {
            (Bound::Unbounded, Bound::Unbounded)
        } else {
            (
                BoundExt::map(range.0, |f| f as usize),
                BoundExt::map(range.1, |f| f as usize).map_raw(|f| match f {
                    Bound::Unbounded => Bound::Unbounded,
                    Bound::Included(x) => Bound::Included(min(x, summaries.len() - 1)),
                    Bound::Excluded(x) => Bound::Excluded(min(x, summaries.len())),
                }),
            )
        };

        tracing::debug!("Transposed to {:?}", xrange);

        //let content: Vec<Summary> =  range.index(&summaries).into();
        // let content = summaries.index(xrange).into();
        let content = summaries[xrange].into();

        Ok(TableOfContents {
            total: self.entries.len() as u64,
            content,
        })
    }

    fn get(&self, id: &Uuid) -> Result<Option<&Recipe>, RepositoryError> {
        Ok(self.entries.get(&id))
    }

    fn remove(&mut self, id: &Uuid) -> Result<(), RepositoryError> {
        self.entries.remove(&id);
        Ok(())
    }

    fn update(&mut self, id: &Uuid, recipe: Recipe) -> Result<UpdateResult, RepositoryError> {
        match self.entries.insert(*id, recipe) {
            Some(_) => Ok(UpdateResult::Changed),
            None => Ok(UpdateResult::Created),
        }
    }
}
