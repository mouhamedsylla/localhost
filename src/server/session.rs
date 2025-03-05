pub mod session {
    use colored::*;
    use uuid::Uuid;
    use std::collections::HashMap;
    use std::time::{SystemTime, Duration};
    use crate::server::errors::{ServerError, SessionError};
    use crate::http::{
        request::Request,
        header::HeaderName,
    };

    #[derive(Debug, Clone)]
    pub struct Session {
        pub id: String,
        pub data: HashMap<String, String>,
        pub created_at: SystemTime,
        pub expires_at: Option<SystemTime>,
    }

    impl Session {
        pub fn new(max_age: Option<u64>) -> Self {
            let now = SystemTime::now();
            let expires_at = max_age.map(|age| now + Duration::from_secs(age));
            Session {
                id: String::new(),
                data: HashMap::new(),
                created_at: now,
                expires_at,
            }
        }
    
        pub fn is_expired(&self) -> bool {
            self.expires_at.map_or(false, |expires| SystemTime::now() > expires)
        }

        pub fn set_id(&mut self, id: String) {
            self.id = id;
        }
    }

    pub trait SessionStore {
        fn get(&self, id: &str) -> Result<Option<Session>, ServerError>;
        fn set(&mut self, session: Session) -> Result<(), ServerError>;
        fn delete(&mut self, id: &str) -> Result<(), ServerError>;
        fn cleanup_expired(&mut self) -> Result<(), ServerError>;
        fn clone_box(&self) -> Box<dyn SessionStore>;
        fn list_sessions(&self) -> Result<Vec<Session>, ServerError>;
        
        fn print_sessions(&self) -> Result<(), ServerError> {
            println!("\n{}", "Current Sessions:".cyan().bold());
            for session in self.list_sessions()? {
                println!("{}", "═".repeat(50).cyan());
                println!("Session ID: {}", session.id.yellow());
                println!("Created at: {}", session.created_at
                    .duration_since(SystemTime::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs());
                if let Some(expires) = session.expires_at {
                    println!("Expires at: {}", expires
                        .duration_since(SystemTime::UNIX_EPOCH)
                        .unwrap_or_default()
                        .as_secs());
                }
                println!("Data: {:?}", session.data);
            }
            println!("{}\n", "═".repeat(50).cyan());
            Ok(())
        }
    }

    impl Clone for Box<dyn SessionStore> {
        fn clone(&self) -> Box<dyn SessionStore> {
            self.clone_box()
        }
    }

    pub mod store_session {
        use super::*;

        #[derive(Debug, Clone)]
        pub struct MemorySessionStore {
            sessions: HashMap<String, Session>
        }

        impl MemorySessionStore {
            pub fn new() -> Self {
                MemorySessionStore {
                    sessions: HashMap::new()
                }
            }
        }

        impl SessionStore for MemorySessionStore {
            fn get(&self, id: &str) -> Result<Option<Session>, ServerError> {
                Ok(self.sessions.get(id).cloned())
            }

            fn set(&mut self, session: Session) -> Result<(), ServerError> {
                self.sessions.insert(session.id.clone(), session);
                Ok(())
            }

            fn delete(&mut self, id: &str) -> Result<(), ServerError> {
                self.sessions.remove(id);
                Ok(())
            }

            fn cleanup_expired(&mut self) -> Result<(), ServerError> {
                self.sessions.retain(|_, session| !session.is_expired());
                Ok(())
            }

            fn clone_box(&self) -> Box<dyn SessionStore> {
                Box::new(self.clone())
            }

            fn list_sessions(&self) -> Result<Vec<Session>, ServerError> {
                Ok(self.sessions.values().cloned().collect())
            }
        }
    }

    pub mod session_manager {
        use super::*;
        use crate::config::config::SessionConfig;
        use crate::http::header::{Header, Cookie, CookieOptions, SameSitePolicy};

        #[derive(Clone)]
        pub struct SessionManager {
            pub config: SessionConfig,
            pub store: Box<dyn SessionStore>,
        }

        impl SessionManager {
            pub fn new(config: SessionConfig, store: Box<dyn SessionStore>) -> Self {
                SessionManager { config, store }
            }

            pub fn create_session(&mut self) -> Result<(Session, Header), ServerError> {
                let option_config = self.config.options.clone();
                let id = generate_id();
                let cookie = if let Some(opts) = option_config {
                    let options = CookieOptions {
                        http_only: opts.http_only.unwrap_or(false),
                        secure: opts.secure.unwrap_or(false),
                        max_age: opts.max_age,
                        path: opts.path,
                        expires: opts.expires.map(|secs| SystemTime::now() + Duration::from_secs(secs)),
                        domain: opts.domain,
                        same_site: match opts.same_site.as_deref().map(str::to_ascii_lowercase).as_deref() {
                            Some("strict") => SameSitePolicy::Strict,
                            Some("lax") => SameSitePolicy::Lax,
                            Some("none") => SameSitePolicy::None,
                            _ => SameSitePolicy::Strict,
                        }
                    };
                    Cookie::with_options(self.config.name.as_deref().unwrap_or(""), &id, options)
                } else {
                    Cookie::new(self.config.name.as_deref().unwrap_or(""), &id)
                };

                let mut session = Session::new(cookie.options.max_age);
                session.set_id(id.clone());

                self.store.set(session.clone())?;
                let header = Header::from_str("set-cookie", &cookie.to_string());
                
                Ok((session, header))
            }

            pub fn get_session(&mut self, cookie_header: Option<&Header>) -> Result<Option<Session>, ServerError> {
                if let Some(header) = cookie_header {
                    if let Some(cookie) = Cookie::parse(&header.value.value) {
                        if let Some(session) = self.store.get(&cookie.value)? {
                            if !session.is_expired() {
                                return Ok(Some(session));
                            }
                            self.store.delete(&cookie.value)?;
                            return Err(SessionError::SessionExpired(cookie.value).into());
                        }
                    }
                }
                Ok(None)
            }

            pub fn destroy_session(&mut self, session_id: &str) -> Result<Header, ServerError> {                
                self.store.delete(session_id)?;
                let mut options = CookieOptions::default();
                options.max_age = Some(0);
                let cookie = Cookie::with_options(
                    self.config.name.as_deref().unwrap_or(""),
                    "",
                    options
                );
                let header = Header::from_str("set-cookie", &cookie.to_string());
                
                self.store.print_sessions()?;
                Ok(header)
            }
        }

        impl Default for SessionManager {
            fn default() -> Self {
                SessionManager {
                    config: SessionConfig::default(),
                    store: Box::new(MemorySessionStore::new()),
                }
            }
        }
    }

    pub mod session_middleware {

        use super::*;
        use crate::server::route::Route;

        pub struct SessionMiddleware {}

        impl SessionMiddleware {
            pub fn process(&self, req: &Request, route: &Route, current_manager: &mut SessionManager) 
                -> Result<Option<Session>, ServerError> {
                
                if let Some(required) = &route.session_required {
                    if !required {
                        return Ok(None);
                    }
                } else {
                    return Ok(None);
                }



                let cookie_header = req.headers.iter().find(|h| h.name == HeaderName::Cookie);

                match current_manager.get_session(cookie_header) {
                    Ok(Some(session)) => {
                        if session.is_expired() {
                            current_manager.destroy_session(&session.id)?;
                            if let Some(redirect_url) = &route.session_redirect {
                                return Err(SessionError::SessionExpired(format!("Redirect to: {}", redirect_url)).into());
                            }
                            return Err(SessionError::SessionExpired(session.id).into());
                        }
                        Ok(Some(session))
                    },
                    Ok(None) => {
                        if let Some(redirect) = &route.session_redirect {
                            return Err(SessionError::SessionExpiredRedirect(redirect.to_string()).into());
                        } else {
                            return Err(SessionError::SessionExpired("Session expired".to_string()).into());
                        }
                    },
                    Err(e) => Err(e),
                }   
            }
        }
    }

    fn generate_id() -> String {
        Uuid::new_v4().to_string()
    }

    pub use session_manager::SessionManager;
    pub use store_session::MemorySessionStore;
    pub use session_middleware::SessionMiddleware;
}