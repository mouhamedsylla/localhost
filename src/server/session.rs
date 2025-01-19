pub mod session {
    use super::*;

    use uuid::Uuid;
    use std::{collections::HashMap};
    use std::time::{SystemTime, Duration};


    #[derive(Debug, Clone)]
    pub struct Session {
        pub id: String,
        pub data: HashMap<String, String>,
        pub created_at: SystemTime,
        pub expires_at: Option<SystemTime>,
    }

    // Session store trait
    pub trait SessionStore: Send + Sync{
        fn get(&self, id: &str) -> Option<Session>;
        fn set(&self, session: Session);
        fn delete(&self, id: &str);
        fn cleanup_expired(&self);
        fn clone_box(&self) -> Box<dyn SessionStore>;
    }

    impl Session {
        pub fn new(max_age: Option<u64>) -> Self {
            let now = SystemTime::now();
            let expires_at = if let Some(max_age) = max_age {
                Some(now + Duration::from_secs(max_age))
            } else {
                None
            };
            Session {
                id: Uuid::new_v4().to_string(),
                data: HashMap::new(),
                created_at: now,
                expires_at
            }
        }
    
        pub fn is_expired(&self) -> bool {
            if let Some(expires_at) = self.expires_at {
                return SystemTime::now() > expires_at;
            }
            false 
        }
    
    }

    impl Clone for Box<dyn SessionStore> {
        fn clone(&self) -> Box<dyn SessionStore> {
            self.clone_box()
        }
    }

    pub mod store_session {
        use super::*;

        #[derive(Debug)]
        pub struct MemorySessionStore {
            sessions: std::sync::RwLock<HashMap<String, Session>>,
        }

        impl MemorySessionStore {
            pub fn new() -> Self {
                MemorySessionStore {
                    sessions: std::sync::RwLock::new(HashMap::new()),
                }
            }
        }

        impl SessionStore for MemorySessionStore {
            fn get(&self, id: &str) -> Option<Session> {
                let sessions = self.sessions.read().unwrap();
                sessions.get(id).cloned()
            }
        
            fn set(&self, session: Session) {
                let mut sessions = self.sessions.write().unwrap();
                sessions.insert(session.id.clone(), session);
            }
        
            fn delete(&self, id: &str) {
                let mut sessions = self.sessions.write().unwrap();
                sessions.remove(id);
            }
        
            fn cleanup_expired(&self) {
                let mut sessions = self.sessions.write().unwrap();
                sessions.retain(|_, session| !session.is_expired());
            }

            fn clone_box(&self) -> Box<dyn SessionStore> {
                Box::new(Self::new())
            }
        }
    }

    pub mod session_manager {
        use std::default;

        use super::*;

        use crate::config::config::SessionConfig;
        use crate::http::header::{Header, Cookie, CookieOptions, SameSitePolicy};

        // Session manager
        #[derive(Clone)]
        pub struct SessionManager {
            pub config: SessionConfig,
            pub store: Box<dyn SessionStore>,
        }

        impl SessionManager {
            pub fn new(config: SessionConfig, store: Box<dyn SessionStore>) -> Self {
                SessionManager { config, store }
            }

            pub fn create_session(&self) -> (Session, Header) {
                let option_config = self.config.options.clone();
                let cookie = if let Some(opts) = option_config {
                    let options = CookieOptions {
                        http_only: opts.http_only.unwrap_or(false),
                        secure: opts.secure.unwrap_or(false),
                        max_age: opts.max_age,
                        path: opts.path,
                        expires: opts.expires.map(|secs| SystemTime::now() + Duration::from_secs(secs)),
                        domain: opts.domain,
                        same_site: if let Some(policy) = opts.same_site {
                            match policy.to_ascii_lowercase().as_str() {
                                "strict" => SameSitePolicy::Strict,
                                "lax" => SameSitePolicy::Lax,
                                "none" => SameSitePolicy::None,
                                _ => SameSitePolicy::Strict
                            }
                        } else {
                            SameSitePolicy::Strict
                        }
                    };
                    Cookie::with_options("", self.config.name.as_deref().unwrap_or(""), options)
                } else {
                    Cookie::new("", self.config.name.as_deref().unwrap_or(""))
                };

                let session = Session::new(cookie.options.max_age);
                let header = Header::from_str("set-cookie", &cookie.to_string());
                (session, header)
            }

            pub fn get_session(&self, cookie_header: Option<&Header>) -> Option<Session> {
                if let Some(header) = cookie_header {
                    if let Some(cookie) = Cookie::parse(&header.value.value) {
                        if cookie.name == self.config.name.as_deref().unwrap_or("") {
                            if let Some(mut session) = self.store.get(&cookie.value) {
                                if !session.is_expired() {
                                    self.store.set(session.clone());
                                    return Some(session);
                                }
                                self.store.delete(&cookie.value);
                            }
                        }
                    }
                }
                None
            }

            pub fn destroy_session(&self, session_id: &str) -> Header {
                self.store.delete(session_id);
                let mut options = CookieOptions::default();
                options.max_age = Some(0);
                let cookie = Cookie::with_options(&self.config.name.as_deref().unwrap_or(""), "", options);
                Header::from_str("set-cookie", &cookie.to_string())
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
        use crate::http::response;
        use crate::http::{
            request::Request,
            response::Response,
            header::{HeaderName, Header},
            status::HttpStatusCode,
            body::Body,
        };
        use crate::server::route::Route;

        pub struct SessionMiddleware {}

        impl SessionMiddleware {

            pub fn process(&self, req: &Request, route: &Route, current_manager: &SessionManager) -> Result<Option<Session>, Response> {
                if let Some(required) = &route.session_required {
                    if !required {
                        return Ok(None);
                    }
                } else {
                    return Ok(None);
                }

                let cookie_header = req.headers.iter().find(|h| h.name == HeaderName::Cookie);

                if let Some(session) = current_manager.get_session(cookie_header) {
                    if session.is_expired() {
                        current_manager.destroy_session(&session.id);
                        if let Some(redirect) = &route.session_redirect {
                            return Err(self.redirect(redirect));
                        }
                        self.session_expire();
                    }
                    Ok(Some(session))                        
                } else {
                    if let Some(redirect) = &route.session_redirect {
                        Err(self.redirect(redirect))
                    } else {
                        Err(self.session_expire())
                    }
                }       
            }

            fn redirect(&self, redirect: &str) -> Response {
                let body = Body::text("Unauthorized: Session required");
                Response::new(
                    HttpStatusCode::Unauthorized,
                    vec![
                        Header::from_str("content-Type", "text/plain"),
                        Header::from_str("content-length", &body.body_len().to_string()),
                        Header::from_str("location", redirect),
                    ],
                    Some(body)
                )
            }

            fn session_expire(&self) -> Response {
                let body = Body::text("Session expired");
                Response::new(
                    HttpStatusCode::Unauthorized,
                    vec![
                        Header::from_str("content-Type", "text/plain"),
                        Header::from_str("content-length", &body.body_len().to_string()),
                    ],
                    Some(body)
                )
            }

        }
    }

    pub use session_manager::SessionManager;
    pub use store_session::MemorySessionStore;
    pub use session_middleware::SessionMiddleware;
}