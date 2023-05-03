use once_cell::sync::Lazy;
use sled::Db;
use std::{collections::HashMap, ops::DerefMut, sync::Mutex};

enum Store {
    DB(Db),
    Map(HashMap<Vec<u8>, Vec<u8>>),
}

// NB: db is automatically closed at end of lifetime
static STORE: Lazy<Mutex<Store>> = Lazy::new(|| {
    Mutex::new(match create_db() {
        Err(err) => {
            crate::log::error!("Failed to create store: {err}");
            Store::Map(HashMap::new())
        }
        Ok(db) => Store::DB(db),
    })
});

fn create_db() -> crate::Result<Db> {
    use std::path::Path;
    let store_path = crate::get_env("STORE_PATH");
    let mut path = if store_path.is_empty() {
        let mut tmp = dirs::home_dir().unwrap_or(Path::new(".").into());
        tmp.push(".chatgpt");
        tmp
    } else {
        Path::new(&store_path).to_path_buf()
    };
    if !path.exists() {
        std::fs::create_dir_all(&path)?;
    }
    path.push("store.sled");
    // 500ms by default
    Ok(sled::Config::new()
        .path(path)
        .flush_every_ms(Some(250))
        .open()?)
}

pub fn put<K, V>(k: K, v: V) -> crate::Result<()>
where
    K: AsRef<[u8]>,
    V: AsRef<[u8]>,
{
    let mut guard = STORE.lock().unwrap();
    match guard.deref_mut() {
        Store::DB(db) => {
            db.insert(k, sled::IVec::from(v.as_ref()))?;
        }
        Store::Map(m) => {
            m.insert(k.as_ref().into(), v.as_ref().into());
        }
    }
    Ok(())
}

pub fn get<K>(k: K) -> crate::Result<Option<Vec<u8>>>
where
    K: AsRef<[u8]>,
{
    let mut guard = STORE.lock().unwrap();
    match guard.deref_mut() {
        Store::DB(db) => Ok(db.get(k)?.map(|x| x.to_vec())),
        Store::Map(m) => Ok(m.get(k.as_ref()).map(|v| v.clone())),
    }
}

pub fn delete<K>(k: K) -> crate::Result<()>
where
    K: AsRef<[u8]>,
{
    let mut guard = STORE.lock().unwrap();
    match guard.deref_mut() {
        Store::DB(db) => {
            db.remove(k)?;
        }
        Store::Map(m) => {
            m.remove(k.as_ref());
        }
    }
    Ok(())
}
