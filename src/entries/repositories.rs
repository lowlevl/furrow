use std::{collections::HashMap, ops::Deref};

use serde::{Deserialize, Serialize};
use serde_with::{serde_as, MapPreventDuplicates};

use super::Entry;
use crate::id::Base;

impl Entry<()> for Repositories {
    const PATH: &'static str = "Repositories.toml";
}

/// An [`Entry`] describing _repositories_ parameters.
#[serde_as]
#[derive(Debug, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Repositories {
    #[serde(default)]
    #[serde_as(as = "MapPreventDuplicates<_, _>")]
    repositories: HashMap<Base, Spec>,
}

impl From<()> for Repositories {
    fn from(_value: ()) -> Self {
        Self::default()
    }
}

/// The configuration for a _repositories_, with some metadata
/// and some technical configuration.
#[serde_as]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Spec {
    pub description: Option<String>,
    pub license: Option<String>,

    #[serde(default)]
    pub visibility: Visibility,
    #[serde_as(as = "Option<serde_with::DisplayFromStr>")]
    pub branches: Option<regex::Regex>,
    #[serde_as(as = "Option<serde_with::DisplayFromStr>")]
    pub tags: Option<regex::Regex>,

    #[serde(default)]
    #[serde_as(as = "MapPreventDuplicates<_, _>")]
    pub branch: HashMap<String, RefConfig>,
}

impl Deref for Repositories {
    type Target = HashMap<Base, Spec>;

    fn deref(&self) -> &Self::Target {
        &self.repositories
    }
}

/// Repository's visibility configuration.
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Visibility {
    /// Only repo owner can clone this repository.
    #[default]
    Private,

    /// Everyone can clone this repository.
    Public,

    /// Everyone can clone this repository, and the repository is read-only.
    Archive,
}

/// Repository's references (_branches, tags_) configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "kebab-case")]
pub struct RefConfig {
    /// Whether _force pushes_ are allowed for this `ref`.
    pub allow_force: bool,

    /// Whether _deletes_ are allowed for this `ref`.
    pub allow_delete: bool,
}

impl RefConfig {
    /// Create a _protected_ [`RefConfig`].
    pub fn protected() -> Self {
        Self {
            allow_force: false,
            allow_delete: false,
        }
    }

    /// Create an _unprotected_ [`RefConfig`].
    pub fn unprotected() -> Self {
        Self {
            allow_force: true,
            allow_delete: true,
        }
    }
}

impl Default for RefConfig {
    fn default() -> Self {
        Self::unprotected()
    }
}
