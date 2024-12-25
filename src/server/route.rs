
#[derive(Debug, Clone)]
pub struct Route {
    pub path: String,
    pub method: String,
   // pub handler: fn(&mut Connection, &str) -> std::io::Result<()>,
}