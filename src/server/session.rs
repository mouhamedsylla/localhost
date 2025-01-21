pub mod session {
    use super::*;
    use colored::*;
    use uuid::Uuid;
    use std::{collections::HashMap, sync::Arc};
    use std::time::{SystemTime, Duration};

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
        fn get(&self, id: &str) -> Option<Session>;
        fn set(&mut self, session: Session);
        fn delete(&mut self, id: &str);
        fn cleanup_expired(&mut self);
        fn clone_box(&self) -> Box<dyn SessionStore>;
        fn list_sessions(&self) -> Vec<Session>;
        
        fn print_sessions(&self) {
            println!("\n{}", "Current Sessions:".cyan().bold());
            for session in self.list_sessions() {
                println!("{}", "═".repeat(50).cyan());
                println!("Session ID: {}", session.id.yellow());
                println!("Created at: {}", session.created_at
                    .duration_since(SystemTime::UNIX_EPOCH)
                    .unwrap()
                    .as_secs());
                if let Some(expires) = session.expires_at {
                    println!("Expires at: {}", expires
                        .duration_since(SystemTime::UNIX_EPOCH)
                        .unwrap()
                        .as_secs());
                }
                println!("Data: {:?}", session.data);
            }
            println!("{}\n", "═".repeat(50).cyan());
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

            fn get(&self, id: &str) -> Option<Session> {
                self.sessions.get(id).cloned()
            }

            fn set(&mut self, session: Session) {
                self.sessions.insert(session.id.clone(), session);
            }

            fn delete(&mut self, id: &str) {
                self.sessions.remove(id);
            }

            fn cleanup_expired(&mut self) {
                self.sessions.retain(|_, session| !session.is_expired());
            }

            fn clone_box(&self) -> Box<dyn SessionStore> {
                Box::new(self.clone())
            }

            fn list_sessions(&self) -> Vec<Session> {
                self.sessions.values().cloned().collect()
            }
         }

    }

    pub mod session_manager {
        use std::default;
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

            pub fn create_session(&mut self) -> (Session, Header) {
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

                self.store.set(session.clone());
                let header = Header::from_str("set-cookie", &cookie.to_string());
                self.store.print_sessions();
                
                (session, header)
            }

            pub fn get_session(&mut self, cookie_header: Option<&Header>) -> Option<Session> {
                if let Some(header) = cookie_header {
                    if let Some(cookie) = Cookie::parse(&header.value.value) {
                        self.store.print_sessions();
                        if let Some(session) = self.store.get(&cookie.value) {
                            if !session.is_expired() {
                                return Some(session);
                            }
                            self.store.delete(&cookie.value);
                        }
                    }
                }
                None
            }

            pub fn destroy_session(&mut self, session_id: &str) -> Header {                
                self.store.delete(session_id);
                let mut options = CookieOptions::default();
                options.max_age = Some(0);
                let cookie = Cookie::with_options(
                    self.config.name.as_deref().unwrap_or(""),
                    "",
                    options
                );
                let header = Header::from_str("set-cookie", &cookie.to_string());
                
                self.store.print_sessions();
                header
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

            pub fn process(&self, req: &Request, route: &Route, current_manager: &mut SessionManager) -> Result<Option<Session>, Response> {
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
                Response::new(
                    HttpStatusCode::Found,
                    vec![Header::from_str("location", redirect)],
                    None
                )
            }

            fn session_expire(&self) -> Response {
                let body = Body::text("Session expired");
                Response::new(
                    HttpStatusCode::Unauthorized,
                    vec![
                        Header::from_str("content-type", "text/plain"),
                        Header::from_str("content-length", &body.body_len().to_string()),
                    ],
                    Some(body)
                )
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