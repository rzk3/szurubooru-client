use crate::models::PagedSearchResult;
use pyo3::exceptions::PyException;
use pyo3::prelude::*;
use pyo3::types::PyList;
use pyo3::types::PyListMethods;

pub mod asynchronous;
pub mod synchronous;

#[derive(Debug)]
#[pyclass(name = "PagedSearchResult", get_all)]
pub struct PyPagedSearchResult {
    pub query: String,
    pub offset: u32,
    pub limit: u32,
    pub total: u32,
    pub results: Py<PyList>,
}

#[cfg_attr(all(feature = "python"), pymethods)]
impl PyPagedSearchResult {
    fn __repr__(&self) -> String {
        format!("{:?}", self)
    }

    /*fn __len__(&self) -> PyResult<usize> {
        Python::with_gil(|py| {
            Ok(self.results.bind_borrowed(py).len())
        })
    }*/
}

impl<T: IntoPy<PyObject>> From<PagedSearchResult<T>> for PyPagedSearchResult {
    fn from(value: PagedSearchResult<T>) -> Self {
        Python::with_gil(|py| {
            let list =
                PyList::new_bound(py, value.results.into_iter().map(|v| v.into_py(py))).unbind();
            PyPagedSearchResult {
                query: value.query,
                offset: value.offset,
                limit: value.limit,
                total: value.total,
                results: list,
            }
        })
    }
}
