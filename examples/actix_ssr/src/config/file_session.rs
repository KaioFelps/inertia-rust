//! See: https://gist.github.com/KaioFelps/a5308ab73f8fa22d268240958ddbd1cb

use std::{collections::HashMap, io, str::FromStr};

use actix_session::storage::{
    generate_session_key, LoadError, SaveError, SessionKey, SessionStore, UpdateError,
};
use chrono::{DateTime, Duration, Utc};

use tokio::{
    fs::{read_dir, read_to_string, remove_file, DirBuilder, DirEntry, File},
    io::{AsyncReadExt, AsyncSeekExt, AsyncWriteExt},
};

const SESSIONS_DIR: &str = "storage/sessions";
const SESSIONS_EXP_KEY: &str = "__expires_at__";

#[derive(Clone)]
pub struct FileSessionStore<'a> {
    sessions_directory: &'a str,
}

pub type SessionState = HashMap<String, String>;

impl Default for FileSessionStore<'_> {
    fn default() -> Self {
        Self {
            sessions_directory: SESSIONS_DIR,
        }
    }
}

#[cfg(test)]
impl<'a> FileSessionStore<'a> {
    fn new(sessions_directory: &'a str) -> Self {
        Self { sessions_directory }
    }
}

impl SessionStore for FileSessionStore<'_> {
    async fn load(&self, session_key: &SessionKey) -> Result<Option<SessionState>, LoadError> {
        match File::open(self.get_session_path(session_key.as_ref())).await {
            Err(_) => Ok(None),
            Ok(mut session) => {
                let mut content = String::new();
                session
                    .read_to_string(&mut content)
                    .await
                    .map_err(Into::into)
                    .map_err(LoadError::Other)?;

                let session: SessionState = serde_json::from_str(&content)
                    .map_err(Into::into)
                    .map_err(LoadError::Deserialization)?;

                if let Some(expiration_date) = session.get(SESSIONS_EXP_KEY) {
                    let has_expired =
                        Self::has_expired(expiration_date).map_err(LoadError::Other)?;

                    if has_expired {
                        return Ok(None);
                    }
                } else {
                    return Ok(None);
                };

                Ok(Some(session))
            }
        }
    }

    async fn save(
        &self,
        session_state: SessionState,
        ttl: &actix_web::cookie::time::Duration,
    ) -> Result<SessionKey, SaveError> {
        self.maybe_create_session_directory().await;

        let session_key = generate_session_key();
        let session_state = self.set_expiration_date(session_state, ttl);

        let session = self
            .serialize_session_state(&session_state)
            .map_err(SaveError::Serialization)?;

        let mut file = File::options()
            .read(true)
            .write(true)
            .create_new(true)
            .open(self.get_session_path(session_key.as_ref()))
            .await
            .map_err(Into::into)
            .map_err(SaveError::Other)?;

        file.write_all(&session)
            .await
            .map_err(Into::into)
            .map_err(SaveError::Other)?;

        file.flush()
            .await
            .map_err(Into::into)
            .map_err(SaveError::Other)?;

        Ok(session_key)
    }

    async fn update(
        &self,
        session_key: SessionKey,
        session_state: SessionState,
        ttl: &actix_web::cookie::time::Duration,
    ) -> Result<SessionKey, UpdateError> {
        let mut file = match File::options()
            .read(true)
            .write(true)
            .open(self.get_session_path(session_key.as_ref()))
            .await
        {
            Err(_) => {
                return self
                    .save(session_state, ttl)
                    .await
                    .map_err(|err| match err {
                        SaveError::Serialization(err) => UpdateError::Serialization(err),
                        SaveError::Other(err) => UpdateError::Other(err),
                    });
            }
            Ok(file) => file,
        };

        let old_session = self.read_and_serialize(&mut file).await;

        let has_expired = !(old_session.is_ok_and(|file| {
            file.get(SESSIONS_EXP_KEY)
                .is_some_and(|exp| Self::has_expired(exp).is_ok_and(|v| !v))
        }));

        if has_expired {
            return self
                .save(session_state, ttl)
                .await
                .map_err(|err| match err {
                    SaveError::Serialization(err) => UpdateError::Serialization(err),
                    SaveError::Other(err) => UpdateError::Other(err),
                });
        }

        let _ = file.set_len(0).await;
        let _ = file.seek(std::io::SeekFrom::End(0)).await;

        let session = self
            .serialize_session_state(&self.set_expiration_date(session_state, ttl))
            .map_err(UpdateError::Serialization)?;

        file.write_all(&session)
            .await
            .map_err(Into::into)
            .map_err(UpdateError::Other)?;

        file.flush()
            .await
            .map_err(Into::into)
            .map_err(UpdateError::Other)?;

        Ok(session_key)
    }

    async fn update_ttl(
        &self,
        session_key: &SessionKey,
        ttl: &actix_web::cookie::time::Duration,
    ) -> Result<(), anyhow::Error> {
        let mut file = match File::options()
            .write(true)
            .read(true)
            .open(self.get_session_path(session_key.as_ref()))
            .await
        {
            Err(_) => {
                return Err(anyhow::Error::msg("Session does not exist."));
            }
            Ok(file) => file,
        };

        let session_state = self.read_and_serialize(&mut file).await?;
        let session_state = self.set_expiration_date(session_state, ttl);
        let session_state = self.serialize_session_state(&session_state)?;

        file.write_all(&session_state)
            .await
            .map_err(Into::into)
            .map_err(UpdateError::Other)?;

        file.flush()
            .await
            .map_err(Into::into)
            .map_err(UpdateError::Other)?;

        Ok(())
    }

    async fn delete(&self, session_key: &SessionKey) -> Result<(), anyhow::Error> {
        remove_file(self.get_session_path(session_key.as_ref()))
            .await
            .map_err(|err| {
                println!("{:#?}", err);
                anyhow::Error::msg("Failed to delete session.")
            })?;

        Ok(())
    }
}

impl FileSessionStore<'_> {
    pub fn get_sessions_dir(&self) -> &str {
        self.sessions_directory
    }

    pub fn get_session_path(&self, session_key: &str) -> String {
        format!("{}/{}.json", self.get_sessions_dir(), session_key)
    }

    fn set_expiration_date(
        &self,
        mut session_state: SessionState,
        ttl: &actix_web::cookie::time::Duration,
    ) -> SessionState {
        let expiration_date = (Utc::now() + Duration::seconds(ttl.whole_seconds())).to_string();
        session_state.insert(SESSIONS_EXP_KEY.into(), expiration_date);
        session_state
    }

    fn serialize_session_state(
        &self,
        session_state: &SessionState,
    ) -> Result<Vec<u8>, anyhow::Error> {
        serde_json::to_vec_pretty(session_state).map_err(Into::into)
    }

    async fn maybe_create_session_directory(&self) {
        if !std::path::Path::new(self.get_sessions_dir()).is_dir() {
            if let Err(err) = DirBuilder::new()
                .recursive(true)
                .create(self.get_sessions_dir())
                .await
            {
                println!(
                    "Session storage does not exist and couldn't be created. Consider creating the directory '{}' yourself. Error: {}",
                    self.get_sessions_dir(),
                    err
                );
            }
        }
    }

    pub fn has_expired(expiration_date: &str) -> Result<bool, anyhow::Error> {
        let expiration_date =
            DateTime::<Utc>::from_str(expiration_date).map_err(Into::<anyhow::Error>::into)?;

        let now = Utc::now();

        Ok(now > expiration_date)
    }

    async fn read_and_serialize(&self, file: &mut File) -> Result<SessionState, anyhow::Error> {
        let mut session_state = String::new();
        file.read_to_string(&mut session_state)
            .await
            .map_err(|err| {
                println!("{:#?}", err);
                anyhow::Error::msg("Failed to read session state.")
            })?;

        serde_json::from_slice(session_state.as_bytes())
            .map_err(|_| anyhow::Error::msg("Failed to serialize session state."))
    }
}

pub async fn clean_expired_sessions() -> io::Result<()> {
    println!("Starting cleanup of expired sessions.");
    inner_clean_expired_sessions(SESSIONS_DIR).await
}

#[inline]
async fn inner_clean_expired_sessions(sessions_dir: &str) -> io::Result<()> {
    let mut directory = read_dir(sessions_dir).await?;

    while let Some(file) = directory.next_entry().await? {
        if file_should_be_cleaned(&file).await {
            println!(
                "Removing session file: '{}'",
                file.path().to_str().unwrap_or("")
            );

            let _ = remove_file(file.path()).await;
        }
    }

    Ok(())
}

async fn file_should_be_cleaned(file: &DirEntry) -> bool {
    let file_path = file.path();

    let file = match read_to_string(file.path()).await {
        Err(err) => {
            println!(
                "Failed to read file {} during garbage collecting. Skiping to next file. Error: {}",
                file_path.to_str().unwrap_or(""),
                err
            );
            return true;
        }
        Ok(file) => file,
    };

    let session: HashMap<String, String> = match serde_json::from_str(&file) {
        Err(err) => {
            println!(
                "Failed to serialize session during garbage collecting. File: {}. Error: {}",
                file_path.to_str().unwrap_or(""),
                err
            );

            return true;
        }
        Ok(session) => session,
    };

    match session.get(SESSIONS_EXP_KEY) {
        None => true,
        Some(exp_date) => FileSessionStore::has_expired(exp_date).unwrap_or(true),
    }
}

#[cfg(test)]
mod tests {
    use super::{inner_clean_expired_sessions, FileSessionStore};
    use actix_session::storage::{generate_session_key, LoadError, SessionStore};
    use actix_web::cookie::time;
    use inertia_rust::hashmap;
    use std::collections::HashMap;
    use tokio::{
        fs::{read_dir, remove_dir_all, File},
        io::AsyncWriteExt,
    };

    async fn write_session(session_key: &str, content: &str, store: &FileSessionStore<'_>) {
        store.maybe_create_session_directory().await;

        let mut file = File::options()
            .read(true)
            .write(true)
            .create_new(true)
            .open(store.get_session_path(session_key))
            .await
            .unwrap();

        file.write_all(content.as_bytes()).await.unwrap();
        file.flush().await.unwrap();
    }

    #[actix_web::test]
    async fn loading_a_missing_session_returns_none() {
        let store = FileSessionStore::default();
        let session_key = generate_session_key();
        assert!(store.load(&session_key).await.unwrap().is_none());
    }

    #[actix_web::test]
    async fn loading_an_invalid_session_state_returns_deserialization_error() {
        let store = FileSessionStore::default();
        store.maybe_create_session_directory().await;

        let session_key = generate_session_key();

        write_session(
            session_key.as_ref(),
            "random-thing-which-is-not-json",
            &store,
        )
        .await;

        assert!(matches!(
            store.load(&session_key).await.unwrap_err(),
            LoadError::Deserialization(_),
        ));
    }

    #[actix_web::test]
    async fn updating_of_an_expired_state_is_handled_gracefully() {
        let store = FileSessionStore::default();
        store.maybe_create_session_directory().await;

        let session_key = generate_session_key();
        let initial_session_key = session_key.as_ref().to_owned();
        let updated_session_key = store
            .update(session_key, HashMap::new(), &time::Duration::seconds(1))
            .await
            .unwrap();

        assert_ne!(initial_session_key, updated_session_key.as_ref());
    }

    #[actix_web::test]
    async fn can_manipulate_non_expired_session() {
        let store = FileSessionStore::default();
        store.maybe_create_session_directory().await;

        let initial_session_key = store
            .save(
                hashmap![
                    "message".into() => "another day another slay".into()
                ],
                &time::Duration::seconds(60),
            )
            .await
            .unwrap();

        let initial_key = initial_session_key.as_ref().to_owned();

        let updated_key = store
            .update(
                initial_session_key,
                hashmap!["message".into() => "a different message.".into()],
                &time::Duration::seconds(60),
            )
            .await
            .unwrap();

        assert_eq!(initial_key, updated_key.as_ref());
    }

    #[actix_web::test]
    async fn cannot_manipulate_expired_but_existing_session() {
        let store = FileSessionStore::default();
        store.maybe_create_session_directory().await;

        let initial_session_key = store
            .save(
                hashmap![
                    "message".into() => "another day another slay".into()
                ],
                &time::Duration::seconds(0),
            )
            .await
            .unwrap();

        let initial_key = initial_session_key.as_ref().to_owned();

        tokio::time::sleep(std::time::Duration::from_millis(100)).await;

        let updated_key = store
            .update(
                initial_session_key,
                hashmap!["message".into() => "a different message.".into()],
                &time::Duration::seconds(60),
            )
            .await
            .unwrap();

        assert_ne!(initial_key, updated_key.as_ref());
    }

    #[actix_web::test]
    async fn garbage_collector_will_remove_expired_sessions_only() {
        let sessions_dir = "storage/sessions/gc";

        // tries to remove the directory if it exists
        let _ = remove_dir_all(sessions_dir).await;

        let store = FileSessionStore::new(sessions_dir);
        store.maybe_create_session_directory().await;

        let _ = store
            .save(
                hashmap![
                    "Foo".into() => "This must be removed.".into()
                ],
                &time::Duration::milliseconds(10),
            )
            .await
            .unwrap();

        let session_key = store
            .save(
                hashmap![
                    "message".into() => "foo".into()
                ],
                &time::Duration::minutes(10),
            )
            .await
            .unwrap();

        tokio::time::sleep(std::time::Duration::from_secs(1)).await;

        let _ = inner_clean_expired_sessions(sessions_dir).await;

        let mut directory = read_dir(sessions_dir).await.unwrap();
        let mut session_files_found = vec![];

        while let Some(file) = directory.next_entry().await.unwrap() {
            session_files_found.push(file.file_name().into_string().unwrap())
        }

        assert_eq!(1, session_files_found.len());
        assert_eq!(
            &[format!("{}.json", session_key.as_ref())],
            session_files_found.as_slice()
        );
    }
}
