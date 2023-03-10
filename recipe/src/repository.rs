use crate::Recipe;
use crate::Summary;
use crate::TableOfContents;
use std::cmp::min;
use std::collections::HashMap;
use std::error;
use std::fmt;
use std::ops::Bound;
use std::ops::RangeBounds;

use uuid::Uuid;
#[derive(Debug, Copy, Clone)]
pub enum Range {
    Empty,
    /// Das Intervall enthält sowohl a als auch b.
    ///
    /// [start, end] = {x | start <= x <= end}
    Closed {
        start: usize,
        end: usize,
    },

    /// [start, +∞) = {x | x >= start}
    LeftClosed {
        start: usize,
    },

    /// (-∞, end] = {x | x <= end}
    RightClosed {
        end: usize,
    },

    /// (-∞, +∞) = N
    Unbounded,
}

impl Range {
    /// Returns the range of a slice specified by self.
    ///
    /// The range is adjusted to the length of the slice.
    ///
    /// # Example
    ///
    /// ```rust
    /// use recipers::repository::Range;
    ///
    /// let numbers = [1, 2, 3, 4, 5];
    /// let range = Range::Closed{start: 2, end: 10};
    /// assert_eq!([3, 4, 5], range.index(&numbers))
    /// ```
    pub fn index<T>(self, slice: &[T]) -> &[T] {
        if slice.len() == 0 {
            return slice;
        }

        match self.clip(slice.len()) {
            Range::Closed { start, end } => &slice[(start..=(min(end, slice.len() - 1)))],
            Range::LeftClosed { start } => &slice[start..],
            Range::RightClosed { end } => &slice[..=(min(end, slice.len() - 1))],
            Range::Unbounded => slice,
            Range::Empty => &slice[0..0],
        }
    }

    fn clip(&self, max_len: usize) -> Range {
        if max_len == 0 {
            return Range::Empty;
        }

        match self {
            Range::Closed { start, end } => Range::Closed {
                start: *start,
                end: min(*end, max_len - 1),
            },
            Range::RightClosed { end } => Range::RightClosed {
                end: min(*end, max_len - 1),
            },
            _ => *self,
        }
    }
}

impl<T> From<&Vec<T>> for Range {
    fn from(value: &Vec<T>) -> Self {
        if value.len() > 0 {
            Range::Closed {
                start: 0,
                end: value.len() - 1,
            }
        } else {
            Range::Empty
        }
    }
}

impl RangeBounds<usize> for Range {
    fn start_bound(&self) -> Bound<&usize> {
        match self {
            Range::Unbounded => Bound::Unbounded,
            Range::RightClosed { .. } => Bound::Unbounded,
            Range::Closed { start, .. } => Bound::Included(start),
            Range::LeftClosed { start } => Bound::Included(start),
            Range::Empty => Bound::Included(&usize::MIN),
        }
    }

    fn end_bound(&self) -> Bound<&usize> {
        match self {
            Range::Unbounded => Bound::Unbounded,
            Range::RightClosed { end } => Bound::Included(end),
            Range::Closed { start: _start, end } => Bound::Included(end),
            Range::LeftClosed { start: _start } => Bound::Unbounded,
            Range::Empty => Bound::Excluded(&usize::MIN),
        }
    }
}

/// An in-memory repository for recipes
pub struct Repository {
    entries: HashMap<Uuid, Recipe>,
}

impl Repository {
    /// Creates a new repository
    pub fn new() -> Repository {
        Repository {
            entries: HashMap::new(),
        }
    }

    /// Adds a recipe to the repository
    pub fn insert(&mut self, r: &Recipe) -> Result<Uuid, RepositoryError> {
        let id = Uuid::new_v4();
        self.entries.insert(id, r.clone());
        Ok(id)
    }

    pub fn list_ids(&self, range: &Range) -> Vec<Uuid> {
        let keys: &Vec<Uuid> = &self.entries.keys().cloned().collect();

        range.index(keys).into()
    }

    /// Creates a table of contents for the specified filter
    /// criteria.
    ///
    /// The recipes are sorted by name. All recipes that start with
    /// "search" are included in the table of contents. The table of
    /// contents contains all the recipes within the given range.
    pub fn list(&self, range: &Range, search: &str) -> Result<TableOfContents, RepositoryError> {
        let mut summaries: Vec<Summary> = self
            .entries
            .iter()
            .map(|entity| entity.into())
            .filter(|s: &Summary| s.title.starts_with(search))
            .collect();

        summaries.sort();
        let content: Vec<Summary> = range.index(&summaries).into();

        Ok(TableOfContents {
            total: self.entries.len(),
            content,
        })
    }

    pub fn get(&self, id: &Uuid) -> Result<Option<&Recipe>, RepositoryError> {
        Ok(self.entries.get(&id))
    }

    pub fn remove(&mut self, id: &Uuid) -> Result<(), RepositoryError> {
        self.entries.remove(&id);
        Ok(())
    }

    pub fn update(&mut self, id: &Uuid, recipe: Recipe) -> Result<UpdateResult, RepositoryError> {
        match self.entries.insert(*id, recipe) {
            Some(_) => Ok(UpdateResult::Changed),
            None => Ok(UpdateResult::Created),
        }
    }
}

#[derive(Debug)]
pub enum RepositoryError {}

impl fmt::Display for RepositoryError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Repository error")
    }
}

impl error::Error for RepositoryError {}

pub enum UpdateResult {
    Changed,
    Created,
}

#[cfg(test)]
mod test {
    use super::{Range, Repository, RepositoryError};
    use crate::Recipe;
    use spucky::spec;

    lazy_static! {
        static ref TESTDATA: Vec<Recipe> = vec![Recipe {
            title: "Lasagne".to_string(),
            preparation: "Du weist schon wie".to_string(),
            servings: 2,
            ingredients: vec![],
        }];
    }

    #[test]
    fn test_insert() -> Result<(), Box<dyn std::error::Error>> {
        let mut repo = Repository::new();

        let recipe = Recipe {
            title: "Lasagne".to_string(),
            preparation: "Du weist schon wie".into(),
            servings: 2,
            ingredients: vec![],
        };

        let id = repo.insert(&recipe)?;

        let copy = repo.get(&id)?;

        assert_eq!(&recipe, copy.unwrap());

        Ok(())
    }

    spec! {
        list_filled_repository {

            case case1 {
                let range = Range::Unbounded;
                let want = 100;
            }

            case case2  {
                let range = Range::LeftClosed { start: 0 };
                let want = 100;
            }

            case case3 {
                let range = Range::RightClosed { end: 99 };
                let want = 100;
            }

            case case4 {
                let range = Range::Closed { start: 0, end: 99 };
                let want=  100;
            }

            case case5 {
                let range = Range::Closed {start: 2, end: 1};
                let want = 0;
            }

            case case6 {
                let range = Range::Closed {start: 99, end: 100};
                let want = 1;
            }

            case case7 {
                let range = Range::Closed {start: 99, end: 99};
                let want = 1;
            }

            case case8 {
                let range = Range::Closed {start: 0, end: 0};
                let want = 1;
            }

            let mut repository = Repository::new();
            fill_with_testdata(&mut repository);

            match repository.list(&range, "") {
                Ok(toc) => assert_eq!(toc.content.len(), want),
                Err(_) => panic!("unexpected error"),
            }
        }

    }

    spec! {
        list_empty_repository {
            case case1 {
                let range = Range::Closed {start: 0, end: 0};
                let want = 0;
            }

            let repository = Repository::new();
            match repository.list(&range, "") {
                Ok(toc) => assert_eq!(toc.content.len(), want),
                Err(_) => panic!("unexpected error",)
            }
        }
    }

    #[test]
    fn list_some_keys() -> Result<(), RepositoryError> {
        let mut repository = Repository::new();
        fill_with_testdata(&mut repository);

        struct Testcase {
            range: Range,
            want: usize,
        }

        let td3: Vec<Testcase> = vec![
            Testcase {
                range: Range::Unbounded,
                want: 100,
            },
            Testcase {
                range: Range::LeftClosed { start: 0 },
                want: 100,
            },
            Testcase {
                range: Range::RightClosed { end: 99 },
                want: 100,
            },
            Testcase {
                range: Range::Closed { start: 0, end: 99 },
                want: 100,
            },
        ];

        for testcase in &td3 {
            let keys = repository.list_ids(&testcase.range);
            assert_eq!(keys.len(), testcase.want)
        }
        Ok(())
    }

    fn fill_with_testdata(repository: &mut Repository) {
        for ele in 0..100 {
            let recipe = Recipe {
                title: format!("Recipe {}", ele),
                preparation: format!("Preparation of recipe {}", ele),
                servings: (ele % 3) + 1,
                ingredients: vec![],
            };
            _ = repository.insert(&recipe);
        }
    }
}
