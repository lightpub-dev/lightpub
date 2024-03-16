use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct PaginatedResponse<T> {
    pub result: Vec<T>,
    pub next: Option<String>,
}

pub trait PaginatableItem {
    type Key;
    fn pkey(&self) -> Self::Key;
}

impl<T> PaginatedResponse<T>
where
    T: PaginatableItem,
{
    pub fn from_result<G>(result: Vec<T>, page_size: usize, next_gen: G) -> Self
    where
        G: FnOnce(&<T as PaginatableItem>::Key) -> String,
    {
        if result.len() <= page_size {
            return Self { result, next: None };
        }
        let last = result.get(page_size).unwrap();
        let next = Some(next_gen(&last.pkey()));
        let result = result.into_iter().take(page_size).collect();
        Self { result, next }
    }
}
