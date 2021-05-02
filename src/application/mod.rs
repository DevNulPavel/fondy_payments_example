use std::{
    sync::{
        Arc
    }
};
use handlebars::{
    Handlebars
};
use reqwest::{
    Client
};
use crate::{
    database::{
        Database
    }
};


#[derive(Debug)]
pub struct AppConfig{
    pub site_url: url::Url,
    pub merchant_id: u64,
    pub merchant_password: String
}

#[derive(Debug)]
pub struct Application{
    pub db: Arc<Database>,
    pub templates: Arc<Handlebars<'static>>,
    pub http_client: Client, // Arc inside
    pub config: Arc<AppConfig>
}