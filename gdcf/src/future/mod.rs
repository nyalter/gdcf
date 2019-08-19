use futures::Future;

pub mod process;
pub mod refresh;
pub mod stream;
pub mod upgrade;

pub trait GdcfFuture: Future {
    type Extension;

    fn cached_extension(&self) -> Option<&Self::Extension>;

    fn has_result_cached(&self) -> bool;
    fn into_cached(self) -> Option<Self::Item>;

    // implementations do this: check if current future is resolvable from cache (if not, return false)
    // if yes, extend -> call closure -> un-extend. If we get our Self::Item from an inner future, wrap
    // this into a closure itself and pass that to __temporarily_extend of that future. The "root"
    // closure comes from a call to has_cached_result and returns `true` if the request is satisfiable
    // from cache, false otherwise. "reverse call stack"
    //fn __temporarily_extend<F: FnOnce(Self::Item) -> bool>(check: F) -> bool;
}