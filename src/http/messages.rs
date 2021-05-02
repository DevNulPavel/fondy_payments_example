use serde::{
    Deserialize,
    Serialize
};
use serde_with::{
    serde_as,
    DisplayFromStr,
    NoneAsEmptyString
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

#[derive(Serialize, Deserialize, Debug)]
pub enum ResponseStatus {
    #[serde(rename = "success")]
    Success,

    #[serde(rename = "failure")]
    Failure
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug, Deserialize)]
pub struct FondyInvalidResponse{
    pub response_status: ResponseStatus,
    pub error_code: i32,
    pub error_message: String
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug, Deserialize)]
pub struct FondyRedirectUrlResponse{
    pub response_status: ResponseStatus,
    pub checkout_url: String, // TODO: Decode as url
    pub payment_id: String
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Serialize, Deserialize, Debug)]
pub enum OrderStatus {
    #[serde(rename = "created")]
    Created,

    #[serde(rename = "processing")]
    Processing,

    #[serde(rename = "declined")]
    Declined,

    #[serde(rename = "approved")]
    Approved,
    
    #[serde(rename = "expired")]
    Expired,

    #[serde(rename = "reversed")]
    Reversed
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Serialize, Deserialize, Debug)]
pub enum TransactionType {
    #[serde(rename = "purchase")]
    Purchase,

    #[serde(rename = "reverse")]
    Reverse
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Serialize, Deserialize, Debug)]
pub enum VerificationStatus {
    #[serde(rename = "verified")]
    Varified,

    #[serde(rename = "incorrect")]
    Incorrect,

    #[serde(rename = "failed")]
    Failed,

    #[serde(rename = "created")]
    Created,
}

////////////////////////////////////////////////////////////////////////////////////////////////////

// TODO: Десереализация строк в enum
// Описание: https://docs.fondy.eu/ru/docs/page/3/
#[serde_as]
#[derive(Debug, Deserialize)]
pub struct FondyPaymentResponse{
    #[serde_as(as = "DisplayFromStr")]
    pub amount: u64,
    
    #[serde_as(as = "NoneAsEmptyString")]
    pub response_code: Option<String>,
    
    #[serde_as(as = "DisplayFromStr")]
    pub reversal_amount: u64,

    #[serde_as(as = "DisplayFromStr")]
    pub settlement_amount: u64,

    #[serde_as(as = "DisplayFromStr")]
    pub actual_amount: u64,

    #[serde_as(as = "DisplayFromStr")]
    pub approval_code: u64,

    pub order_id: String,
    pub merchant_id: u64,
    pub currency: String,
    pub order_status: OrderStatus,
    pub response_status: ResponseStatus,
    pub signature: String,
    pub tran_type: TransactionType,
    pub sender_cell_phone: String,
    pub sender_account: String,
    pub masked_card: String,
    pub card_bin: u64,
    pub card_type: String,
    pub rrn: String,
    pub response_description: String,
    pub settlement_currency: String,
    pub order_time: String,
    pub settlement_date: String,
    pub eci: String,
    pub fee: String,
    pub payment_system: String,
    pub sender_email: String,
    pub payment_id: u32,
    pub actual_currency: String,
    pub product_id: String,
    pub merchant_data: String,
    pub verification_status: String, // VerificationStatus,
    pub rectoken: String,
    pub rectoken_lifetime: String,
    // pub additional_info: serde_json::Value
}