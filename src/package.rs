pub struct Package {
    pub name: String,
    pub version: String,
    pub description: String,
    pub dependencies: Option<Vec<String>>,
}
