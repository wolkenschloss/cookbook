use crate::Recipe;
use crate::Summary;
use crate::TableOfContents;
use std::collections::HashMap;
use std::ops::Range;

use std::slice::SliceIndex;

use uuid::Uuid;

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

    pub fn list2(&self, range: &dyn SliceIndex<[Uuid], Output = [Uuid]>) -> Vec<Uuid> {
        let keys = &self.entries.keys().cloned().collect::<Vec<Uuid>>();
        range.get(&keys);
        let some_keys = &keys[range];
        some_keys.into()
    }

    pub fn list(
        &self,
        range: Range<usize>,
        search: &str,
    ) -> Result<TableOfContents, RepositoryError> {
        let mut sumvec: Vec<Summary> = (&self.entries)
            .iter()
            .map(|entity| entity.into())
            .filter(|s: &Summary| s.title.starts_with(search))
            .collect();

        sumvec.sort();

        let page = &sumvec[range];
        Ok(TableOfContents {
            total: self.entries.len(),
            content: page.to_vec(),
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
    use std::{process::Output, slice::SliceIndex};

    use crate::Recipe;

    use super::{Repository, RepositoryError};

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

    use std::collections::HashSet;
    use std::ops::{Range, RangeFull};
    use uuid::Uuid;

    type UuidSliceIndex = dyn SliceIndex<[Uuid], Output = [Uuid]>;

    #[test]
    fn list_some_keys() -> Result<(), RepositoryError> {
        let mut repository = Repository::new();
        fill_with_testdata(&mut repository);

        struct Testcase {
            range: Box<dyn SliceIndex<[Uuid], Output = [Uuid]>>,
            want: usize,
        }

        let mut td3: Vec<Testcase> = Vec::new();
        let tdd3 = Testcase {
            range: Box::new(..),
            want: 100,
        };
        td3.push(tdd3);

        let tdd4 = Testcase {
            range: Box::new(0..),
            want: 100,
        };
        td3.push(tdd4);

        for testcase in &td3 {
            let r = *(testcase.range);
            let keys = repository.list2(r.copy());
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

    #[test]
    fn test_chatgpt() {
        let vec = vec![1, 2, 3, 4, 5];
        let open_range = 1..;
        let half_open_range = ..4;
        let slice1 = get_slice_from_range(&vec, open_range.start..);
        let slice2 = get_slice_from_range(&vec, ..half_open_range.end);

        println!("{:?}", slice1); // Output: [2, 3, 4, 5]
        println!("{:?}", slice2); // Output: [1, 2, 3, 4]
    }

    fn get_slice_from_range<'a, I>(vec: &'a [i32], range: I) -> &'a I::Output
    where
        I: std::slice::SliceIndex<[i32]>,
    {
        &vec[range]
    }
}
