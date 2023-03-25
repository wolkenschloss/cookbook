use axum::{http::StatusCode, response::IntoResponse};
use std::{
    cmp::min,
    error, fmt,
    ops::{Bound, RangeBounds, Sub},
};
use uuid::Uuid;

use crate::{Recipe, TableOfContents};

#[cfg(feature = "ephemeral")]
pub mod memory;

#[cfg(all(not(feature = "ephemeral"), feature = "mongodb"))]
pub mod mongodb;

pub trait Repository {
    fn insert(&mut self, r: &Recipe) -> Result<Uuid, RepositoryError>;
    fn insert_all(&mut self, recipes: &[Recipe]) -> Result<Vec<Uuid>, RepositoryError>;
    fn list(
        &self,
        range: &(Bound<u64>, Bound<u64>),
        search: &str,
    ) -> Result<TableOfContents, RepositoryError>;

    fn get(&self, id: &Uuid) -> Result<Option<Recipe>, RepositoryError>;
    fn remove(&mut self, id: &Uuid) -> Result<(), RepositoryError>;
    fn update(&mut self, id: &Uuid, recipe: &Recipe) -> Result<UpdateResult, RepositoryError>;
}

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

trait BoundExt<T> {
    fn map<U, F: FnOnce(T) -> U>(self, f: F) -> Bound<U>;
    fn map_raw<U, F: FnOnce(Bound<T>) -> Bound<U>>(self, f: F) -> Bound<U>;
}

impl<T> BoundExt<T> for Bound<T>
where
    T: Ord + Sub,
{
    fn map<U, F: FnOnce(T) -> U>(self, f: F) -> Bound<U> {
        match self {
            Bound::Unbounded => Bound::Unbounded,
            Bound::Included(x) => Bound::Included(f(x)),
            Bound::Excluded(x) => Bound::Excluded(f(x)),
        }
    }

    fn map_raw<U, F: FnOnce(Bound<T>) -> Bound<U>>(self, f: F) -> Bound<U> {
        f(self)
    }
}

#[derive(Debug)]
pub enum RepositoryError {
    Poison,
    MongoDb,
}

impl IntoResponse for RepositoryError {
    fn into_response(self) -> axum::response::Response {
        let body = "internal server error: code rot 7";
        (StatusCode::INTERNAL_SERVER_ERROR, body).into_response()
    }
}

impl fmt::Display for RepositoryError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Repository error")
    }
}

impl error::Error for RepositoryError {}

#[derive(PartialEq, Debug)]
pub enum UpdateResult {
    Changed,
    Created,
}

#[cfg(test)]
mod test {

    use std::ops::Bound;

    use super::Repository;

    use crate::Recipe;

    use serial_test::serial;

    use spucky::spec;
    lazy_static! {
        static ref TESTDATA: Vec<Recipe> = vec![Recipe {
            title: "Lasagne".to_string(),
            preparation: "Du weist schon wie".to_string(),
            servings: 2,
            ingredients: vec![],
        }];
    }

    #[cfg(feature = "ephemeral")]
    fn create_repository() -> impl Repository {
        super::memory::Repository::new()
    }

    #[cfg(all(not(feature = "ephemeral"), feature = "mongodb"))]
    fn create_repository() -> impl Repository {
        let result = super::mongodb::MongoDbClient::new();
        let client = result.unwrap();
        client.collection.drop(None).expect("db must be empty");
        client
    }

    #[test]
    #[serial]
    fn test_insert() -> Result<(), Box<dyn std::error::Error>> {
        let mut repo = create_repository(); // Repository::new();

        let recipe = Recipe {
            title: "Lasagne".to_string(),
            preparation: "Du weist schon wie".into(),
            servings: 2,
            ingredients: vec![],
        };

        let id = repo.insert(&recipe)?;

        let copy = repo.get(&id)?;

        assert_eq!(recipe, copy.unwrap());

        Ok(())
    }

    spec! {
        #[serial]
        list_filled_repository {

            case case1 {
                let range = (Bound::Unbounded, Bound::Unbounded);
                let want = 100;
            }

            case case2  {
                let range = (Bound::Included(0), Bound::Unbounded);
                let want = 100;
            }

            case case3 {
                let range = (Bound::Unbounded, Bound::Included(99));
                let want = 100;
            }

            case case4 {
                let range = (Bound::Included(0), Bound::Included(99));
                let want=  100;
            }

            case case5 {
                let range = (Bound::Included(2), Bound::Included(1));
                let want = 0;
            }

            case case6 {
                let range = (Bound::Included(99), Bound::Included(100));
                let want = 1;
            }

            case case7 {
                let range = (Bound::Included(99), Bound::Included(99));
                let want = 1;
            }

            case case8 {
                let range = (Bound::Included(0), Bound::Included(0));
                let want = 1;
            }

            case case9 {
                let range = (Bound::Included(10), Bound::Excluded(20));
                let want = 10;
            }

            case case10 {
                let range = (Bound::Included(10), Bound::Included(19));
                let want = 10;
            }

            let mut repository = create_repository();// Repository::new();
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
                let range = (Bound::Included(0), Bound::Included(0));
                let want = 0;
            }

            let repository = create_repository();// Repository::new();
            match repository.list(&range, "") {
                Ok(toc) => assert_eq!(toc.content.len(), want),
                Err(_) => panic!("unexpected error",)
            }
        }
    }

    spec! {
        #[ignore]
        update_recipe {
            type Output = Result<(), Box<dyn std::error::Error>>;
            case update {
                let recipe = Recipe {
                    title: "Lasagne".to_string(),
                    preparation: "Du weist schon wie".to_string(),
                    servings: 4,
                    ingredients: vec![] };
                    let mut repository = create_repository();
                    let id = repository.insert(&recipe)?;
                    let want = crate::repository::UpdateResult::Changed;
            }

            case create {
                    let id = uuid::Uuid::new_v4();
                    let mut repository = create_repository();
                    let want = crate::repository::UpdateResult::Created;
            }

            let chili = Recipe {
                title: "Chili con carne".to_string(),
                preparation: "kochen".to_string(),
                servings: 3,
                ingredients: vec![],
            };

            let result = repository.update(&id, &chili)?;
            let changed = repository.get(&id)?;

            assert_eq!(result, want);
            assert_eq!(changed, Some(chili));

            Ok(())
        }
    }

    spec! {
        delete_recipe {
            type Output = Result<(), Box<dyn std::error::Error>>;

            case existing {
                let mut repository = create_repository();
                let lasagne = Recipe {
                    title: "Lasagne".to_string(),
                    preparation: "Du weist schon wie".to_string(),
                    servings: 4,
                    ingredients: vec![],
                };
                let id = repository.insert(&lasagne)?;
            }

            case missing {
                let mut repository = create_repository();
                let id = uuid::Uuid::new_v4();
            }

            repository.remove(&id)?;

            let result = repository.get(&id)?;
            assert_eq!(result, None);

            Ok(())
        }
    }

    fn fill_with_testdata(repository: &mut impl Repository) {
        for ele in 0..100 {
            let recipe = Recipe {
                title: format!("Recipe {}", ele),
                preparation: format!("Preparation of recipe {}", ele),
                servings: (ele % 3) + 1,
                ingredients: vec![],
            };

            _ = repository.insert(&recipe).unwrap();
        }
    }

    #[test]
    fn unbound_range_experiment() {
        let data = [1i32, 2, 3, 4, 5];
        let range = (Bound::Unbounded, Bound::Unbounded);

        let got = &data[range];
        assert_eq!(got, &data[..])
    }

    #[test]
    fn len_as_index_experiment() {
        let data = [1, 2, 3, 4, 5];
        let got = &data[..data.len()];
        assert_eq!(&[1, 2, 3, 4, 5], got);
    }
}
