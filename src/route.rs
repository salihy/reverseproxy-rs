
#[derive(serde::Deserialize, Debug, Clone)]

pub struct Route {
    pub upstream_path: String,
    pub downstream_path: String,
    pub downstream_method: String,
    pub downstream_uri: String,
}

#[derive(serde::Deserialize, Debug, Clone)]
pub struct Routes {
    pub routes: Vec::<Route>
}