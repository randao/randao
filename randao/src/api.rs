use serde::Serialize;
use std::borrow::Cow;

use actix_web::{
    body::BoxBody, error, http::StatusCode, Error, HttpRequest, HttpResponse, Responder,
    ResponseError,
};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ApiResult<T = ()> {
    pub code: i32,
    pub msg: Option<Cow<'static, str>>,
    pub data: Option<T>,
}

impl<T: Serialize> ApiResult<T> {
    pub fn new() -> Self {
        Self {
            code: 200,
            msg: None,
            data: None,
        }
    }
    pub fn code(mut self, code: i32) -> Self {
        self.code = code;
        self
    }
    pub fn with_msg<S: Into<Cow<'static, str>>>(mut self, msg: S) -> Self {
        self.msg = Some(msg.into());
        self
    }
    pub fn msg_as_str(&self) -> &str {
        self.msg.as_ref().map(|s| s.as_ref()).unwrap_or_default()
    }
    pub fn with_data(mut self, data: T) -> Self {
        self.data = Some(data);
        self
    }
    pub fn log_to_resp(&self, req: &HttpRequest) -> HttpResponse {
        self.log(req);
        self.to_resp()
    }
    pub fn log(&self, req: &HttpRequest) {
        info!(
            "{} \"{} {} {:?}\" {}",
            req.peer_addr().unwrap(),
            req.method(),
            req.uri(),
            req.version(),
            self.code
        );
    }
    pub fn to_resp(&self) -> HttpResponse {
        let resp = match serde_json::to_string(self) {
            Ok(json) => HttpResponse::Ok()
                .content_type("application/json")
                .body(json),
            Err(e) => Error::from(e).into(),
        };

        resp
    }
}

use std::fmt::{self, Debug, Display};
use log::info;

impl<T: Debug + Serialize> Display for ApiResult<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

pub type ApiError = ApiResult<()>;
impl<T: Debug + Serialize> ResponseError for ApiResult<T> {
    fn status_code(&self) -> StatusCode {
        StatusCode::OK
    }
    fn error_response(&self) -> HttpResponse {
        self.to_resp()
    }
}

// Either and AsRef/Responder not in crate
pub enum ApiRt<L, R> {
    Ref(L),
    T(R),
}

impl<T, R> Responder for ApiRt<R, ApiResult<T>>
where
    T: Serialize,
    R: AsRef<ApiResult<T>>,
{
    type Body = BoxBody;

    fn respond_to(self, req: &HttpRequest) -> HttpResponse {
        match self {
            ApiRt::Ref(a) => a.as_ref().respond_to(req),
            ApiRt::T(b) => b.respond_to(req),
        }
    }
}

impl<T: Serialize> Responder for ApiResult<T> {
    type Body = BoxBody;

    fn respond_to(self, req: &HttpRequest) -> HttpResponse {
        (&self).respond_to(req)
    }
}
impl<T: Serialize> Responder for &ApiResult<T> {
    type Body = BoxBody;

    fn respond_to(self, req: &HttpRequest) -> HttpResponse {
        self.log_to_resp(req)
    }
}

// return 200 all
pub fn json_error_handler<E: std::fmt::Display + std::fmt::Debug + 'static>(
    err: E,
    req: &HttpRequest,
) -> error::Error {
    let detail = err.to_string();
    let api = ApiResult::new().with_data(()).code(400).with_msg(detail);
    let response = api.log_to_resp(req);

    error::InternalError::from_response(err, response).into()
}

pub async fn notfound(req: HttpRequest) -> Result<HttpResponse, Error> {
    let api = ApiResult::new()
        .with_data(())
        .code(404)
        .with_msg("route not found");

    Ok(api.respond_to(&req))
}
