use serde::{
    Deserialize
};

/*
/// Специальный шаблонный тип, чтобы можно было парсить возвращаемые ошибки в ответах.
/// А после этого - конвертировать в результаты.
#[derive(Deserialize, Debug)]
#[serde(untagged)]
pub enum DataOrErrorResponse<D, E>{
    Ok(D),
    Err(E)
}
impl<D, E> DataOrErrorResponse<D, E> {
    pub fn into_result(self) -> Result<D, E> {
        match self {
            DataOrErrorResponse::Ok(ok) => Ok(ok),
            DataOrErrorResponse::Err(err) => Err(err),
        }
    }
}*/

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Deserialize, Debug)]
pub struct FondyResponse<D>{
    pub response: D
}
impl<D> FondyResponse<D> {
    pub fn into_response(self) -> D {
        self.response
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

/// Специальный шаблонный тип, чтобы можно было парсить возвращаемые ошибки в ответах.
/// А после этого - конвертировать в результаты.
#[derive(Deserialize, Debug)]
#[serde(untagged)]
pub enum FondyDataOrErrorResponse<D, E>{
    Ok(FondyResponse<D>),
    Err(FondyResponse<E>)
}
impl<D, E> FondyDataOrErrorResponse<D, E> {
    pub fn into_result(self) -> Result<D, E> {
        match self {
            FondyDataOrErrorResponse::Ok(ok) => Ok(ok.into_response()),
            FondyDataOrErrorResponse::Err(err) => Err(err.into_response()),
        }
    }
}


////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug, Deserialize)]
pub struct FondyInvalidResponse{
    pub response_status: String,
    pub error_code: i32,
    pub error_message: String
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug, Deserialize)]
pub struct FondyRedirectUrlResponse{
    pub response_status: String,
    pub checkout_url: String,
    pub payment_id: String
}