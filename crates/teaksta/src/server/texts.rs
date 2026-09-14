//! Where a kept text lives, and the one seam every reader and writer of one
//! goes through.
//!
//! A teacher who asks for their text to be kept is asking for an address that
//! still works next term. A file under a directory on the pod does not: the
//! container image is rebuilt, the pod is rescheduled, and the text is gone.
//! So a kept text goes to a store that outlives the process — Azure Blob
//! Storage, which is what the university runs — and the address handed back
//! names this deployment rather than a disk.
//!
//! # The two backings, and having neither
//!
//! [`TextStore::from_config`] reads the deployment. All three of
//! [`AZURE_ACCOUNT_ENV`], [`AZURE_CONTAINER_ENV`] and [`AZURE_ACCESS_KEY_ENV`]
//! set is a deployment that stores in Azure; a named keep directory is one
//! that stores under it. One or two of the three is neither, and fails the
//! boot: a deployment that meant to store in Azure and quietly wrote to a
//! directory that is deleted with the pod is the failure this whole seam
//! exists to remove, and it is invisible until a teacher comes back for a text
//! that is not there.
//!
//! Naming **neither** is a deployment that stores no texts, and that is a
//! configuration rather than an accident: it takes no uploads, serves no
//! stored text, and tells its client so. What it does not do is accept a
//! teacher's text and put it somewhere it will not survive.
//!
//! Both backings are `object_store`'s, so the code above this seam is the
//! same either way and a test, a laptop and the model suites run with no Azure
//! at all. The local backing is rooted at the keep directory the deployment
//! named, so a deployment that never had Azure keeps its texts exactly where
//! it kept them.
//!
//! Azure is reached through `object_store`'s own shared-key signing rather
//! than through an Azure SDK: the `azure_storage_blobs` stack a Rust service
//! would otherwise be written against is frozen on its vendor's legacy branch.
//! What that buys here is that the store adds no second HTTP client and no
//! second crypto library to a process that already has one of each.
//!
//! # Keys
//!
//! A key is the content's own digest, and nothing a teacher typed reaches it.
//! The uploaded filename is read by the media-type gate and then dropped, so
//! there is no name to escape a container prefix with, no name to collide with
//! another teacher's, and no name to disclose.
//!
//! Content addressing is also what makes the store's immutability free. The
//! same text stored twice is one object, and a key that is already there can
//! only be there with these exact bytes — so a write that finds one is a
//! write that has already happened rather than a conflict to resolve. What a
//! key names never changes, which is the whole of the guarantee the stored
//! file's `0400` mode used to carry.

use std::path::{Path, PathBuf};
use std::sync::Arc;

use anyhow::{Context as _, Result};
use object_store::azure::MicrosoftAzureBuilder;
use object_store::local::LocalFileSystem;
use object_store::path::Path as ObjectPath;
use object_store::{
    Error as StoreError, ObjectStore, ObjectStoreExt as _, PutMode, PutOptions, PutPayload,
};
use tracing::warn;

use crate::context::{
    AZURE_ACCESS_KEY_ENV, AZURE_ACCOUNT_ENV, AZURE_CONTAINER_ENV, Config, UPLOAD_KEEP_DIR_ENV,
};
use crate::server::fetch::{Oversized, Unreachable};

/// How many hex characters a stored text's name carries. 128 bits: past the
/// reach of a search for two texts sharing one, and past the reach of a search
/// for any name at all, which is what keeps the addresses this deployment
/// hands out from being enumerable.
const TEXT_ID_LEN: usize = 32;

/// The route a kept text is reachable at, which is also how an address that
/// names one is recognised. It is written once here and read by the router,
/// the upload endpoint and the address vetting alike, so the three cannot
/// disagree about what a text reference looks like.
pub const TEXTS_PATH: &str = "/api/texts/";

/// No text is stored under that name. Reaches a client as a 404 whether they
/// asked for the text itself or named it as a page to enhance: a text that is
/// not there is not there, whoever asked.
#[derive(Debug, Clone, thiserror::Error)]
#[error("no stored text is named {0}")]
pub struct Missing(pub String);

/// The name of one stored text: [`TEXT_ID_LEN`] lowercase hex characters and
/// nothing else.
///
/// Held apart from `String` on purpose. It is the only thing that becomes an
/// object key, and it can only be built by [`TextId::of`] — which derives it
/// from content — or by [`TextId::parse`], which admits the same alphabet.
/// So no separator, no `..`, no percent-encoded anything and no caller's
/// string reaches the store, and the checks that would otherwise have to be
/// repeated at the endpoint, at the vetting seam and at the store are made
/// once, here.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TextId(String);

impl TextId {
    /// The name the given bytes are stored under. The digest is
    /// cryptographic, so two texts sharing a name is not something an
    /// uploader can arrange — which matters, because a name that could be
    /// collided is a name a teacher's text could be replaced under.
    pub fn of(content: &[u8]) -> Self {
        let digest = blake3::hash(content);
        TextId(digest.to_hex()[..TEXT_ID_LEN].to_string())
    }

    /// The name a caller's string holds, when it holds one at all.
    pub fn parse(raw: &str) -> Option<Self> {
        let named = raw.len() == TEXT_ID_LEN
            && raw
                .bytes()
                .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte));
        named.then(|| TextId(raw.to_string()))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// The address this deployment hands out for the text, which is the
    /// address it also reads back. It carries no host: the deployment's own
    /// name is not something a request can be trusted about, and a reference
    /// with nothing to spoof cannot be spoofed. See
    /// [`crate::server::fetch::target`], which is where that matters.
    pub fn reference(&self) -> String {
        format!("{TEXTS_PATH}{}", self.0)
    }

    fn key(&self) -> ObjectPath {
        ObjectPath::from(self.0.as_str())
    }
}

impl std::fmt::Display for TextId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

/// What the deployment stores kept texts in.
#[derive(Clone)]
pub struct TextStore {
    store: Arc<dyn ObjectStore>,
    backing: Backing,
}

/// Which of the two backings is in force, for the startup line an operator
/// reads and for the one thing the local backing still does for itself.
#[derive(Debug, Clone, PartialEq, Eq)]
enum Backing {
    /// A container in Azure Blob Storage, named by account and container. The
    /// key is not held: it is the builder's, and it is not this type's to
    /// hand back to anything that formats itself.
    Azure { account: String, container: String },
    /// A directory on this machine.
    Local(PathBuf),
}

// Hand-written, because the derived one on a store holding a shared account
// key would print it into any log line or panic message that formatted the
// state. `Backing` holds no key, so what is shown is the account, the
// container and the directory — which is what an operator needs to see.
impl std::fmt::Debug for TextStore {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TextStore")
            .field("backing", &self.backing)
            .finish()
    }
}

impl TextStore {
    /// The store this deployment was configured for, or `None` for a
    /// deployment that was configured with none.
    ///
    /// Azure wins over a named keep directory when both are there. The
    /// directory is the switch a laptop turns the flow on with, and the
    /// container is what a deployment means by storage when it has one: a
    /// pod that carries both is a pod whose emptyDir was left in the manifest
    /// beside the secret somebody has since minted, and writing a teacher's
    /// text to the emptyDir because it was named too would be reading the
    /// leftover as the intent.
    ///
    /// # Errors
    ///
    /// A deployment whose Azure settings are incomplete, or whose keep
    /// directory will not open as a store root.
    // [spec:teaksta:def:sme.src.main.java.werti.server.upload-download-file-servlet.upload-download-file-servlet.text-store-fn+1]
    // [spec:teaksta:sem:sme.src.main.java.werti.server.upload-download-file-servlet.upload-download-file-servlet.text-store-fn+1]
    pub fn from_config(config: &Config) -> Result<Option<Self>> {
        match (&config.azure, config.upload_keep_dir.as_deref()) {
            (Some(azure), named) => {
                if let Some(directory) = named {
                    warn!(
                        "{} names an Azure container and {UPLOAD_KEEP_DIR_ENV} names {}; \
                         kept texts go to Azure",
                        AZURE_ACCOUNT_ENV,
                        directory.display()
                    );
                }
                Self::azure(azure).map(Some)
            }
            (None, Some(directory)) => Self::local(directory).map(Some),
            (None, None) => Ok(None),
        }
    }

    /// A container in Azure Blob Storage. The builder opens no socket, so a
    /// deployment finds out that its container is unreachable when a text is
    /// stored rather than at boot — which is the right way round for a
    /// service whose other endpoints do not need the store at all.
    fn azure(azure: &AzureStorage) -> Result<Self> {
        let store = MicrosoftAzureBuilder::new()
            .with_account(&azure.account)
            .with_container_name(&azure.container)
            .with_access_key(&azure.access_key)
            .build()
            .context("building the Azure blob client")?;
        Ok(TextStore {
            store: Arc::new(store),
            backing: Backing::Azure {
                account: azure.account.clone(),
                container: azure.container.clone(),
            },
        })
    }

    /// The keep directory, as a store. The directory is the one the
    /// deployment already configured and the boot already created, so a
    /// deployment that never had Azure keeps its texts where it kept them and
    /// the addresses the `file:` era minted still resolve.
    fn local(directory: &Path) -> Result<Self> {
        // The boot creates this directory, and creating it again is free; what
        // it buys is that a store opens from any configuration that names one,
        // rather than only from the configuration the boot built.
        std::fs::create_dir_all(directory)
            .with_context(|| format!("creating {}", directory.display()))?;
        let store = LocalFileSystem::new_with_prefix(directory)
            .with_context(|| format!("opening {} as a text store", directory.display()))?;
        Ok(TextStore {
            store: Arc::new(store),
            backing: Backing::Local(directory.to_path_buf()),
        })
    }

    /// What this deployment stores kept texts in, for the startup report.
    /// Names the account and container, never the key.
    pub fn describe(&self) -> String {
        match &self.backing {
            Backing::Azure { account, container } => {
                format!("the Azure container {container} on the account {account}")
            }
            Backing::Local(directory) => format!("the directory {}", directory.display()),
        }
    }

    /// Whether this deployment's kept texts outlive the pod they were stored
    /// from. The local backing is a directory, and a directory on a
    /// container's filesystem is gone with the container.
    pub fn is_durable(&self) -> bool {
        matches!(self.backing, Backing::Azure { .. })
    }

    /// Stores a text and hands back the name it is now reachable under.
    ///
    /// The write is create-only, which is the whole of the immutability
    /// guarantee: what a name holds is what it held when it was first
    /// written. A name already taken is not an error, because the name is the
    /// content's digest — the object that is there is these bytes — so the
    /// second teacher to offer the same text is answered with the same
    /// address rather than a conflict.
    // [spec:teaksta:def:sme.src.main.java.werti.server.upload-download-file-servlet.upload-download-file-servlet.text-store-fn+1]
    // [spec:teaksta:sem:sme.src.main.java.werti.server.upload-download-file-servlet.upload-download-file-servlet.text-store-fn+1]
    pub async fn put(&self, content: Vec<u8>) -> Result<TextId> {
        let id = TextId::of(&content);
        let options = PutOptions {
            mode: PutMode::Create,
            ..PutOptions::default()
        };

        match self
            .store
            .put_opts(&id.key(), PutPayload::from(content), options)
            .await
        {
            Ok(_) => {
                self.seal(&id)?;
                Ok(id)
            }
            // Already stored, under a name that can only mean these bytes.
            Err(StoreError::AlreadyExists { .. }) => Ok(id),
            Err(error) => Err(anyhow::Error::new(error).context(format!("storing the text {id}"))),
        }
    }

    /// Reads a stored text, up to `cap` bytes.
    ///
    /// The weight is settled before any of it is held: the store reports an
    /// object's size with its body, so a text over the cap is refused rather
    /// than read and then thrown away. Nothing this deployment stores can be
    /// over it — the upload gate refuses anything that weighs more — so an
    /// object over the cap names a container holding something this
    /// deployment did not put there, and reading it whole is not the way to
    /// find that out.
    // [spec:teaksta:def:sme.src.main.java.werti.server.upload-download-file-servlet.upload-download-file-servlet.text-store-fn+1]
    // [spec:teaksta:sem:sme.src.main.java.werti.server.upload-download-file-servlet.upload-download-file-servlet.text-store-fn+1]
    // [spec:teaksta:sem:sme.src.main.java.werti.server.wer-ti-servlet.wer-ti-servlet.page-cap-fn+1]
    pub async fn get(&self, id: &TextId, cap: usize) -> Result<Vec<u8>> {
        let reference = id.reference();
        let read = self
            .store
            .get(&id.key())
            .await
            .map_err(|error| match error {
                StoreError::NotFound { .. } => anyhow::Error::new(Missing(id.to_string())),
                other => anyhow::Error::new(other).context(Unreachable(reference.clone())),
            })?;

        if read.meta.size > cap as u64 {
            return Err(Oversized {
                address: reference,
                cap,
            }
            .into());
        }

        let bytes = read
            .bytes()
            .await
            .map_err(|error| anyhow::Error::new(error).context(Unreachable(reference.clone())))?;
        // The size the store announced is what was checked, so a store that
        // announced one size and delivered another is caught rather than
        // trusted.
        if bytes.len() > cap {
            return Err(Oversized {
                address: reference,
                cap,
            }
            .into());
        }
        Ok(bytes.to_vec())
    }

    /// Puts a locally stored text into the owner-read-only mode the keep
    /// directory has always held its texts in.
    ///
    /// The store's create-only write is what guarantees immutability, and it
    /// guarantees it for both backings. This is the local backing's second
    /// lock on the same door: a directory is reachable by everything else
    /// running as this user, which a container is not, so the mode that used
    /// to be the whole guarantee is still worth setting where there is a mode
    /// to set. An Azure object has none and needs none.
    fn seal(&self, id: &TextId) -> Result<()> {
        let Backing::Local(directory) = &self.backing else {
            return Ok(());
        };
        // A key is hex, so it is one path component and it joins to exactly
        // the file the local store just wrote.
        crate::server::upload::set_read_only(&directory.join(id.as_str()))
    }
}

/// The three settings that name an Azure container, read together because one
/// or two of them is not a deployment.
#[derive(Clone, PartialEq, Eq)]
pub struct AzureStorage {
    pub account: String,
    pub container: String,
    /// The shared account key. Secret material: never logged, never echoed,
    /// and kept out of every rendering of the configuration it sits in.
    pub access_key: String,
}

// Hand-written for the same reason [`TextStore`]'s is: a derived one would
// print the account key into the startup report, which prints the whole
// configuration.
impl std::fmt::Debug for AzureStorage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AzureStorage")
            .field("account", &self.account)
            .field("container", &self.container)
            .field("access_key", &"<redacted>")
            .finish()
    }
}

impl AzureStorage {
    /// The three names, in the order an operator reading a failure wants
    /// them.
    pub const VARIABLES: [&'static str; 3] =
        [AZURE_ACCOUNT_ENV, AZURE_CONTAINER_ENV, AZURE_ACCESS_KEY_ENV];
}

#[cfg(test)]
#[path = "texts_tests.rs"]
mod tests;
