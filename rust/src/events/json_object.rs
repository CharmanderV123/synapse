/*
 * This file is licensed under the Affero General Public License (AGPL) version 3.
 *
 * Copyright (C) 2026 Element Creations Ltd
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU Affero General Public License as
 * published by the Free Software Foundation, either version 3 of the
 * License, or (at your option) any later version.
 *
 * See the GNU Affero General Public License for more details:
 * <https://www.gnu.org/licenses/agpl-3.0.html>.
 *
 */

use std::{collections::BTreeMap, sync::Arc};

use pyo3::{
    exceptions::PyKeyError,
    pyclass, pymethods,
    types::{PyIterator, PyMapping},
    Bound, IntoPyObject, PyAny, PyResult, Python,
};
use pythonize::{depythonize, pythonize};
use serde::{Deserialize, Serialize};

/// A generic class for representing immutable JSON objects.
///
/// This is used for representing the `content` field of an event.
///
/// The basic architecture here is to optimize for two things:
/// 1. Fast access of top-level keys (e.g. `event.content["key"]`)
/// 2. Pure Rust implementation.
#[derive(Serialize, Deserialize)]
#[pyclass(mapping, frozen)]
#[derive(Clone)]
#[serde(transparent)]
pub struct JsonObject {
    object: Arc<BTreeMap<Box<str>, serde_json::Value>>,
}

#[pymethods]
impl JsonObject {
    #[new]
    #[pyo3(signature = (content = None))]
    fn new<'a, 'py>(content: Option<&'a Bound<'py, PyAny>>) -> PyResult<Self> {
        let content_dict = match content {
            Some(content) => depythonize(content)?,
            None => BTreeMap::new(),
        };

        Ok(Self {
            object: Arc::new(content_dict),
        })
    }

    fn __len__(&self) -> usize {
        self.object.len()
    }

    fn __contains__(&self, key: &str) -> bool {
        self.object.contains_key(key)
    }

    fn __getitem__<'py>(&self, py: Python<'py>, key: &str) -> PyResult<Bound<'py, PyAny>> {
        let Some(value) = self.object.get(key) else {
            return Err(PyKeyError::new_err(key.to_string()));
        };
        Ok(pythonize(py, value)?)
    }

    fn __iter__<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyIterator>> {
        PyIterator::from_object(
            &self
                .object
                .keys()
                .map(|k| &**k)
                .collect::<Vec<_>>()
                .into_pyobject(py)?,
        )
    }

    fn keys<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        self.object
            .keys()
            .map(|k| &**k)
            .collect::<Vec<_>>()
            .into_pyobject(py)
    }

    fn values<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        let values: Vec<_> = self
            .object
            .values()
            .map(|v| pythonize(py, v))
            .collect::<Result<Vec<_>, _>>()?;
        values.into_pyobject(py)
    }

    fn items<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        let items: Vec<_> = self
            .object
            .iter()
            .map(|(k, v)| PyResult::Ok((k.as_ref(), pythonize(py, v)?)))
            .collect::<PyResult<Vec<_>>>()?;
        items.into_pyobject(py)
    }

    #[pyo3(signature = (key, default=None))]
    fn get<'py>(
        &self,
        py: Python<'py>,
        key: &str,
        default: Option<Bound<'py, PyAny>>,
    ) -> PyResult<Bound<'py, PyAny>> {
        match self.object.get(key) {
            Some(value) => Ok(pythonize(py, value)?),
            None => Ok(default.into_pyobject(py)?),
        }
    }

    fn __eq__(&self, other: Bound<'_, PyMapping>) -> bool {
        let Ok(other_dict) = depythonize(&other) else {
            return false;
        };

        *self.object == other_dict
    }

    fn __str__(&self) -> String {
        serde_json::to_string(&self.object)
            .expect("Value should be serializable")
            .to_string()
    }

    fn __repr__(&self) -> String {
        serde_json::to_string(&self.object)
            .expect("Value should be serializable")
            .to_string()
    }
}
