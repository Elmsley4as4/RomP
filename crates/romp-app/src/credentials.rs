//! Where the app's secrets live.

pub trait TokenStore: Send + Sync {
    fn load(&self, server: &str) -> Option<String>;
    fn save(&self, server: &str, token: &str) -> Result<(), String>;
    fn delete(&self, server: &str);
}

/// Keeps sign-ins in the app's own store, not in the login keychain.
///
/// A keychain item is handed only to an app with the same code identity that
/// created it, so a changed bundle id or signing certificate turns the next read
/// into an "enter your login keychain password" dialog. Nobody using the app can
/// answer that, the app cannot suppress it, and clearing the item needs the same
/// authorisation — so tokens live in romp.db instead, beside the server URL and
/// the library, readable by this user account only.
pub struct StoredTokens {
    store: std::sync::Arc<std::sync::Mutex<crate::store::Store>>,
    prefix: &'static str,
}

impl StoredTokens {
    /// The RomM sign-in, keyed by server URL.
    pub fn new(store: std::sync::Arc<std::sync::Mutex<crate::store::Store>>) -> Self {
        Self::with_prefix(store, "romm_token")
    }

    /// Any other sign-in, keyed by account name.
    pub fn with_prefix(
        store: std::sync::Arc<std::sync::Mutex<crate::store::Store>>,
        prefix: &'static str,
    ) -> Self {
        Self { store, prefix }
    }

    fn key(&self, account: &str) -> String {
        format!("{}:{}", self.prefix, account)
    }
}

impl TokenStore for StoredTokens {
    fn load(&self, server: &str) -> Option<String> {
        let token = self.store.lock().ok()?.get(&self.key(server))?;
        (!token.is_empty()).then_some(token)
    }

    fn save(&self, server: &str, token: &str) -> Result<(), String> {
        let store = self
            .store
            .lock()
            .map_err(|_| "The library store is locked.".to_string())?;
        store.set(&self.key(server), token);
        Ok(())
    }

    fn delete(&self, server: &str) {
        if let Ok(store) = self.store.lock() {
            store.remove(&self.key(server));
        }
    }
}

#[cfg(test)]
#[derive(Default)]
pub struct MemoryTokens(std::sync::Mutex<std::collections::HashMap<String, String>>);

#[cfg(test)]
impl TokenStore for MemoryTokens {
    fn load(&self, server: &str) -> Option<String> {
        self.0.lock().unwrap().get(server).cloned()
    }
    fn save(&self, server: &str, token: &str) -> Result<(), String> {
        self.0.lock().unwrap().insert(server.into(), token.into());
        Ok(())
    }
    fn delete(&self, server: &str) {
        self.0.lock().unwrap().remove(server);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn shared_store() -> std::sync::Arc<std::sync::Mutex<crate::store::Store>> {
        std::sync::Arc::new(std::sync::Mutex::new(
            crate::store::Store::open_in_memory().unwrap(),
        ))
    }

    #[test]
    fn the_sign_in_lives_in_the_store_not_the_keychain() {
        let store = shared_store();
        let tokens = StoredTokens::new(store.clone());
        assert_eq!(tokens.load("http://192.168.3.7:3020/"), None);
        tokens.save("http://192.168.3.7:3020/", "token").unwrap();
        assert_eq!(
            tokens.load("http://192.168.3.7:3020/").as_deref(),
            Some("token")
        );
        // Per server, and stored where a re-sign cannot take it away.
        assert_eq!(tokens.load("http://other/"), None);
        assert_eq!(
            store
                .lock()
                .unwrap()
                .get("romm_token:http://192.168.3.7:3020/"),
            Some("token".to_string())
        );
        tokens.delete("http://192.168.3.7:3020/");
        assert_eq!(tokens.load("http://192.168.3.7:3020/"), None);
    }

    #[test]
    fn each_kind_of_sign_in_keeps_its_own_keys() {
        let store = shared_store();
        let romm = StoredTokens::new(store.clone());
        let achievements = StoredTokens::with_prefix(store.clone(), "ra_token");
        romm.save("http://a/", "romm").unwrap();
        achievements.save("a", "ra").unwrap();
        assert_eq!(romm.load("a"), None);
        assert_eq!(achievements.load("http://a/"), None);
        assert_eq!(romm.load("http://a/").as_deref(), Some("romm"));
        assert_eq!(achievements.load("a").as_deref(), Some("ra"));
        achievements.delete("a");
        assert_eq!(romm.load("http://a/").as_deref(), Some("romm"));
    }

    #[test]
    fn an_empty_token_reads_as_signed_out() {
        let store = shared_store();
        store.lock().unwrap().set("romm_token:http://a/", "");
        assert_eq!(StoredTokens::new(store).load("http://a/"), None);
    }

    #[test]
    fn memory_tokens_are_per_server() {
        let t = MemoryTokens::default();
        t.save("http://a/", "one").unwrap();
        t.save("http://b/", "two").unwrap();
        assert_eq!(t.load("http://a/").as_deref(), Some("one"));
        t.delete("http://a/");
        assert_eq!(t.load("http://a/"), None);
        assert_eq!(t.load("http://b/").as_deref(), Some("two"));
    }
}
