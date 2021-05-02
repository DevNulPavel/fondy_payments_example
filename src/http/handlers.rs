use std::{
    sync::{
        Arc
    }
};
use tracing::{
    debug, 
    error, 
    instrument
};
use warp::{
    Filter,
    Reply,
    Rejection,
    reject::{
        Reject
    }
};
use serde::{
    Deserialize
};
use serde_json::{
    json
};
use tap::{
    prelude::{
        *
    }
};
use reqwest_inspect_json::{
    InspectJson
};
use crate::{
    error::{
        FondyError
    },
    application::{
        Application,
        AppConfig
    }
};
use super::{
    messages::{
        FondyDataOrErrorResponse,
        FondyInvalidResponse,
        FondyRedirectUrlResponse,
        FondyPaymentResponse
    },
    signature::{
        calculate_signature
    }
};

//////////////////////////////////////////////////////////////////////////////////////////

impl Reject for FondyError {
}

//////////////////////////////////////////////////////////////////////////////////////////

#[instrument(skip(app))]
async fn index(app: Arc<Application>) -> Result<impl Reply, Rejection>{
    let html = app
        .templates
        .render("index", &json!({}))
        .map_err(FondyError::from)
        .tap_err(|err| { error!("Index template rendering failed: {}", err); })?;

    Ok(warp::reply::html(html))
}

//////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug, Deserialize)]
struct BuyItemParams{
    item_id: i32
}

// Передаем сюда лишь конфиг и клиента, а не все приложение для возможности тестирования
#[instrument(skip(http_client, config))]
async fn buy(http_client: reqwest::Client, config: Arc<AppConfig>, buy_params: BuyItemParams) -> Result<impl Reply, Rejection>{
    debug!("Buy params: {:#?}", buy_params);

    let order_id = uuid::Uuid::new_v4().to_string();

    // Стоимость в центах, то есть умноженная на 10
    let price: i32 = 10*10;

    // Стоимость в центах, то есть умноженная на 10
    let currency = "RUB";

    // Адрес, куда будет редиректиться браузер
    let browser_redirect_url = config
        .site_url
        .join("browser_redirect_callback_url")
        .map_err(FondyError::from)
        .tap_err(|err| { error!("Url join error: {}", err); })?;
    debug!("Browser callback url: {}", browser_redirect_url);

    // Коллбека на нашем сервере
    let server_callback_url = config
        .site_url
        .join("purchase_server_callback")
        .map_err(FondyError::from)
        .tap_err(|err| { error!("Url join error: {}", err); })?;
    debug!("Server callback url: {}", server_callback_url);

    // Данные, которые будут в коллбеке
    // let callback_data = "our_custom_payload";

    // Идентификатор нашего продукта
    let product_id = format!("{}", buy_params.item_id);

    // Все параметры, но без подписи
    // TODO: В структуру сериализации
    let mut parameters = json!({
        "order_id": order_id,
        "merchant_id": config.merchant_id, 
        "order_desc": "My product description",
        "amount": price,
        "currency": currency,
        "version": "1.0.1",
        // "merchant_data": callback_data,
        "server_callback_url": server_callback_url.as_str(),
        "response_url": browser_redirect_url.as_str(),
        "product_id": product_id
        // "payment_systems": "card, banklinks_eu, banklinks_pl",
        // "default_payment_system": "card",
        // "lifetime": 36000,
        // "preauth": "N" // Тип снятия денег
        // "sender_email": "test@gmail.com"
        // "delayed": "Y"
        // "lang": "ru"
        // "required_rectoken": "N"         // Получение токена для будущих автоматических оплат
        // "rectoken": "AAAA"               // Токен, по которому можно будет автоматически списывать деньги потом
        // "receiver_rectoken": "AAAA"      // Токен карты, по которому можно кредитовать карту, не передавая полный номер карты
        // "verification": "N"
        // "verification_type": "amount"
        // "design_id"                      // Кастомный дизайн
        // "subscription"                   // Подписка на периодические платежи
        // "subscription_callback_url"      // URL коллбека, куда будет перенаправлен покупатель при периодической покупке
    });

    // Вычисляем подпись и добавляем к параметрам
    let signature = calculate_signature(&config.merchant_password, &parameters, "signature")
        .tap_err(|err| { error!("Signature calculate error: {}", err); })?;
    parameters["signature"] = serde_json::Value::String(signature);

    debug!("Fondy request params: {:#?}", &parameters);

    // Параметры: https://docs.fondy.eu/ru/docs/page/3/
    let response = http_client
        .post("https://pay.fondy.eu/api/checkout/url")
        .json(&json!({
            "request": parameters
        }))
        .send()
        .await
        .map_err(FondyError::from)
        .tap_err(|err|{ error!("Fondy request send failed: {}", err); })?
        .inspect_json::<FondyDataOrErrorResponse<FondyRedirectUrlResponse, FondyInvalidResponse>,
                        FondyError>(|data|{
            debug!("Fondy received data: {}", data)
        })
        .await
        .tap_err(|err| { error!("Fondy response parsing failed: {}", err); })?
        .into_result()
        .map_err(FondyError::from)
        .tap_err(|err| { error!("Fondy fail response: {:#?}", err); })?;

    debug!("Received reponse: {:#?}", response);

    // Возвращаем код 307 + POST параметры
    use std::str::FromStr;
    let uri = warp::http::Uri::from_str(response.checkout_url.as_str())
        .map_err(FondyError::from)
        .tap_err(|err| { error!("Invaid receive URI: {:#?}", err); })?;

    Ok(warp::redirect::see_other(uri))
}

//////////////////////////////////////////////////////////////////////////////////////////

#[instrument(skip(data), fields(order_id = %data.order_id, order_status = ?data.order_status))]
async fn purchase_server_callback(data: FondyPaymentResponse) -> Result<impl Reply, Rejection>{
    debug!("Purchase server callback success! Data: {:#?}", data);

    // Данный коллбек вызывается несколько раз на изменение статуса платежа

    // - Проверяем сигнатура на основании пароля
    // - Проверяем, не была ли выдача уже через базу с транзакцией
    // - Оповещаем наш сервер
    // - Если наш сервер не ответил, тогда ставим в очередь периодическую отправку оповещения + сохраняем в базу до подтверждения
    
    // Может быть сразу делать коллбек на наш сервер для выдачи??

    Ok(warp::reply())
}

//////////////////////////////////////////////////////////////////////////////////////////

// #[instrument(skip(data), fields(order_id = %data.order_id, order_status = ?data.order_status))]
// async fn browser_callback(data: FondyPaymentResponse) -> Result<impl Reply, Rejection>{
#[instrument]
async fn browser_callback() -> Result<impl Reply, Rejection>{
    Ok(warp::reply::html("Success"))
}

//////////////////////////////////////////////////////////////////////////////////////////

#[instrument]
async fn rejection_to_json(rejection: Rejection) -> Result<impl Reply, Rejection> {
    if let Some(err) = rejection.find::<FondyError>(){
        let reply = warp::reply::json(&json!({
            "code": warp::http::StatusCode::INTERNAL_SERVER_ERROR.as_u16(),
            "message": err.to_string()
        }));
        Ok(warp::reply::with_status(reply, warp::http::StatusCode::INTERNAL_SERVER_ERROR))
    }else{
        Err(rejection)
    }
}

//////////////////////////////////////////////////////////////////////////////////////////

pub async fn start_server(app: Arc<Application>) {
    // Маршрут индекса
    let index = warp::path::end()
        .and(warp::get())    
        .and(warp::any().map({
            let index_app = app.clone();
            move || { 
                index_app.clone()
            }
        }))
        .and_then(index);

    // Маршрут для покупки
    let buy = warp::path::path("buy")
        .and(warp::post())
        .and(warp::any().map({
            let http_client = app.http_client.clone();
            move || { 
                http_client.clone()
            }
        }))
        .and(warp::any().map({
            let config = app.config.clone();
            move || { 
                config.clone()
            }
        }))
        .and(warp::filters::body::form())
        .and_then(buy)
        .recover(rejection_to_json);

    // Маршрут для коллбека после покупки
    let purchase_server_cb = warp::path::path("purchase_server_callback")
        .and(warp::post())
        .and(warp::filters::body::json()) // Коллбеки POST + Json
        .and_then(purchase_server_callback);

    // Маршрут для коллбека после покупки
    let purchase_browser_cb = warp::path::path("browser_redirect_callback_url")
        .and(warp::post())
        // .and(warp::filters::body::form()) // В браузере POST + Form
        .and_then(browser_callback);

    let routes = index
        .or(buy)
        .or(purchase_server_cb)
        .or(purchase_browser_cb);

    warp::serve(routes)
        .bind(([0, 0, 0, 0], 8080))
        .await;
}

//////////////////////////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod tests{

    #[tokio::test]
    async fn test_buy_redirect(){
        // TODO:
    }
}