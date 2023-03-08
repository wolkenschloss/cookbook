use crate::Recipe;
use crate::Summary;
use crate::TableOfContents;
use std::collections::HashMap;

use uuid::Uuid;

pub enum Range {
    /// Das Intervall enthält sowohl a als auch b.
    ///
    /// [a,b]:=\{x\in \mathbb{R} \mid a\leq x\leq b\}
    /// [a, b] = {x | a <= x <= b}
    Closed { start: usize, end: usize },

    /// [a, +∞) = {x | x >= a}
    LeftClosed { start: usize },

    /// (-∞, b] = {x | x <= b}
    RightClosed { end: usize },

    /// (-∞, +∞) = R
    Unbounded,
}

impl Range {
    fn get<'a, T>(&self, slice: &'a [T]) -> &'a [T] {
        match self {
            Range::Closed { start, end } => &slice[(*start..=*end)],
            Range::LeftClosed { start } => &slice[(*start..)],
            Range::RightClosed { end } => &slice[(..=*end)],
            Range::Unbounded => slice,
        }
    }
}

// impl SliceIndex<Uuid> for Range {
//     type Output;

//     fn get(self, slice: &Uuid) -> Option<&Self::Output> {
//         todo!()
//     }

//     fn get_mut(self, slice: &mut Uuid) -> Option<&mut Self::Output> {
//         todo!()
//     }

//     unsafe fn get_unchecked(self, slice: *const Uuid) -> *const Self::Output {
//         todo!()
//     }

//     unsafe fn get_unchecked_mut(self, slice: *mut Uuid) -> *mut Self::Output {
//         todo!()
//     }

//     fn index(self, slice: &Uuid) -> &Self::Output {
//         todo!()
//     }

//     fn index_mut(self, slice: &mut Uuid) -> &mut Self::Output {
//         todo!()
//     }
// }
pub struct Repository {
    entries: HashMap<Uuid, Recipe>,
}

impl Repository {
    pub fn new() -> Repository {
        Repository {
            entries: HashMap::new(),
        }
    }

    /// Fügt ein Rezept in das Repository ein
    pub fn insert(&mut self, r: &Recipe) -> Result<Uuid, RepositoryError> {
        let id = Uuid::new_v4();
        self.entries.insert(id, r.clone());
        Ok(id)
    }

    pub fn list_ids(&self, range: &Range) -> Vec<Uuid> {
        let keys = &self.entries.keys().cloned().collect::<Vec<Uuid>>();

        range.get(keys).into()
    }

    pub fn list(&self, range: Range, search: &str) -> Result<TableOfContents, RepositoryError> {
        let mut sumvec: Vec<Summary> = (&self.entries)
            .iter()
            .map(|entity| entity.into())
            .filter(|s: &Summary| s.title.starts_with(search))
            .collect();

        sumvec.sort();
        let res: Vec<Summary> = range.get(&sumvec).into();

        Ok(TableOfContents {
            total: self.entries.len(),
            content: res,
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

pub enum UpdateResult {
    Changed,
    Created,
}

#[cfg(test)]
mod test {
    use super::{Range, Repository, RepositoryError};
    use crate::Recipe;

    lazy_static! {
        static ref TESTDATA: Vec<Recipe> = vec![Recipe {
            title: "Lasagne".to_string(),
            preparation: "Du weist schon wie".to_string(),
            servings: 2,
            ingredients: vec![],
        }];
    }

    #[test]
    fn test_insert() -> Result<(), RepositoryError> {
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

    // #[test]
    // fn test_list() -> Result<(), RepositoryError> {
    //     let mut repository = Repository::new();
    //     for d in TESTDATA.iter() {
    //         let _ = repository.insert(d)?;
    //     }

    //     repository.list2(1..);
    //     repository.list2(..4);

    //     let toc = repository.list(.., "Las")?;
    //     // println!("{:?}", toc);

    //     Ok(())
    // }
}
