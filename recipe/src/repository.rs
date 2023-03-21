use axum::{http::StatusCode, response::IntoResponse};
use std::{
    cmp::min,
    error, fmt,
    ops::{Bound, RangeBounds, Sub},
};

pub mod memory;
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

pub enum UpdateResult {
    Changed,
    Created,
}

#[cfg(test)]
mod test {
    use std::ops::Bound;

    use super::memory::Repository;
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
                let range = (Bound::Included(0), Bound::Included(0));
                let want = 0;
            }

            let repository = Repository::new();
            match repository.list(&range, "") {
                Ok(toc) => assert_eq!(toc.content.len(), want),
                Err(_) => panic!("unexpected error",)
            }
        }
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

    #[test]
    fn unbound_range_experiment() {
        let data = [1i32, 2, 3, 4, 5];
        let range = (Bound::Unbounded, Bound::Unbounded);

        let got = &data[range];
        assert_eq!(got, &data[..])
    }

    // #[test]
    // fn include_range_experiment() {
    //     let data = [1, 2, 3, 4, 5];
    //     let range = (Bound::<u64>::Unbounded, Bound::Included(5u64));

    //     let got = &data[range];

    //     assert_eq!(got, &data[..=3])
    // }

    #[test]
    fn len_as_index_experiment() {
        let data = [1, 2, 3, 4, 5];
        let got = &data[..data.len()];
        assert_eq!(&[1, 2, 3, 4, 5], got);
    }
}
