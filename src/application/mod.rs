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
pub struct Application{
    pub db: Database,
    pub templates: Handlebars<'static>,
    pub http_client: Client,
    pub site_url: url::Url
}