use quick_error::{
    quick_error
};

quick_error!{
    #[derive(Debug)]
    pub enum FondyError {
        RequestError(err: reqwest::Error){
            from()
        }

        JsonParseError(err: serde_json::Error){
            from()
        }

        TemplateRenderError(err: handlebars::RenderError){
            from()
        }

        UrlError(err: url::ParseError){
            from()
        }

        InvalidAPIResponse(err: crate::http::FondyInvalidResponse){
            from()
        }

        URIParsingFailed(err: warp::http::uri::InvalidUri){
            from()
        }

        SignatureCalculateError(desc: String){
        }

        UTF8ParseError(err: std::str::Utf8Error){
            from()
        }

        Custom(desc: String){
        }
    }
}



