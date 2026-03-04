use axum::{http::{HeaderMap, StatusCode}, response::{IntoResponse, Response}};
use jwt_authentification::{Cookie, Duration as CookieMaxLife};
use serde::Serialize;
use thiserror::Error;
use tracing::error;

#[derive(Error, Debug)]
pub enum Error 
{
    #[error(transparent)]
    DeserializeError(#[from] serde_json::Error),
    #[error(transparent)]
    IoError(#[from] std::io::Error),
    #[error(transparent)]
    SqlxError(#[from] sqlx::Error),
    #[error("Ошибка авторизации: `{0}`")]
    AuthError(String),
    #[error("Ошибка инициализации настроек: `{0}`")]
    ConfigurationError(String),
	#[error("Время сессии закончилось, необходимо зайти в систему заново")]
	SessionExpired,
    #[error("Пользователь с такими данными не найден")]
	UserNotFound,
    #[error("Пользователь с таким именем пользователя уже существует")]
	UserExists,
    #[error("Ошибка обновления данных")]
	UserUpdateError,
    #[error("Сессия не найдена")]
	SessionNotFound,
    #[error("Неопознан тип для даункастинга sql запроса")]
	SqlTypeDowncastError,
    #[error("Введенный интервал дат уже занят другим состоянием, введите другой диапазон")]
    DateIntervalNotAllowed,
    #[error("Ваш код для верификации был просрочен, попробуйте еще раз")]
	VerificationCodeExpired,
    #[error("Неверный код верификации, попробуйте еще раз")]
	VerificationCodeWrong,
    #[error("Запись верификации контакта не обнаружена, попробуйте запросить верификацию повторно")]
	VerificationNotFound,
    #[error("Ошибка пароля `{0}`")]
	WrongPassword(String),
    #[error("Ошибка при конвертации файла `{0}`")]
	FileConvertion(String),
    #[error("Ошибка имени пользователя")]
	WrongUsername,
    #[error("Ошибка формата входных данных")]
	InputDataError,
    #[error("Ошибка выгрузки файлов `{0}`")]
	UploadError(String),
    #[error("Ошибка загрузки файла: `{0}`")]
    DownloadError(String),
    #[error(transparent)]
    JwtError(#[from] jwt_authentification::JwtError),
    #[error("Отпечаток сессии не совпадает, сессия будет удалена, необходимо зайти заново")]
    WrongFingerprintError(String),
    #[error("Уникальный идетификатор клиента не найден или имеет неверный формат")]
    FingerprintNotFound,
    #[error("Ошибка при обработке поля `{0}` (tiberius)")]
    TiberiusFromRowError(String),
    #[error(transparent)]
    UtilitesError(#[from] utilites::error::Error),
    #[error("Ошибка, такая роль не существует")]
    RoleParseError,
    #[error("Ошибка создания VNode из html")]
    HtmlToVnodeError,
    #[error("{0}")]
    NotFound(String),
    #[error("Ошибка аргументов: {0}")]
    ArgumentsError(String),
    #[error("У вас нехватает прав для получения информации")]
    AccessDeniedError,
    #[error("Не найдено соотвествия хеша загруженного документа и образа расположенного в директории документов НТЦ")]
    ImageHashNotFound,
    #[error("Такого эндпоинта не существует")]
    PathNotFound,
    #[error("Ошибка валидации входных данных: {0}")]
    ValidationError(String),
    #[error("Ошибка получения данных формы: {0}")]
    AxumMultipartError(String)

}

impl serde::Serialize for Error 
{
	fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
	where
	S: serde::ser::Serializer,
	{
		serializer.serialize_str(self.to_string().as_ref())
	}
}

impl IntoResponse for Error
{
    fn into_response(self) -> axum::response::Response 
    {
        let message = self.to_string();
        match self
        {
            Error::WrongFingerprintError(cookie_name) =>
            {
                cookie_remove_error_response(&message, &cookie_name)
            },
            Error::SqlxError(e) => 
            {
                error!("{:?}", e);
                let body = String::from("Ошибка базы данных");
                ServerErrorResponse::new(StatusCode::INTERNAL_SERVER_ERROR, body).into_response()
            },
            Error::UploadError(u) =>
            {
                let body = u;
                ServerErrorResponse::new(StatusCode::INTERNAL_SERVER_ERROR, body).into_response()
            },
            Error::VerificationCodeExpired =>
            {
                let body = self.to_string();
                ServerErrorResponse::new(StatusCode::REQUEST_TIMEOUT, body).into_response()
            },
            Error::ImageHashNotFound =>
            {
                let body = self.to_string();
                ServerErrorResponse::new(StatusCode::NOT_FOUND, body).into_response()
            },
            Error::PathNotFound =>
            {
                let body = self.to_string();
                ServerErrorResponse::new(StatusCode::NOT_FOUND, body).into_response()
            },
            Error::DownloadError(_) =>
            {
                let body = self.to_string();
                ServerErrorResponse::new(StatusCode::FORBIDDEN, body).into_response()
            },
            Error::AccessDeniedError =>
            {
                let body = self.to_string();
                ServerErrorResponse::new(StatusCode::FORBIDDEN, body).into_response()
            },
            Error::WrongUsername =>
            {
                let body = self.to_string();
                ServerErrorResponse::new(StatusCode::NOT_ACCEPTABLE, body).into_response()
            },
            Error::DateIntervalNotAllowed =>
            {
                let body = self.to_string();
                ServerErrorResponse::new(StatusCode::NOT_ACCEPTABLE, body).into_response()
            },
            Error::IoError(e) =>
            {
                let body = e.to_string();
                error!("{:?}", &body);
                ServerErrorResponse::new(StatusCode::BAD_REQUEST, body).into_response()
            },
            _ => 
            {
                let body = self.to_string();
                error!("{:?}", &body);
                ServerErrorResponse::new(StatusCode::BAD_REQUEST, body).into_response()
            }
        }
    }
}

#[derive(Serialize, Debug, Clone)]
pub struct ServerErrorResponse
{
    err: String
}
impl ServerErrorResponse
{
    pub fn new(status_code: StatusCode, error: String) -> impl IntoResponse
    {
        let e = Self {err: error};
        (status_code, serde_json::to_string(&e).unwrap())
    }
    pub fn access_denied<T: ToString>(body: T) -> impl IntoResponse
    {
        let err = body.to_string();
        (StatusCode::FORBIDDEN, err)
    }
}
//TODO подправить ответ со структурой ServerErrorResponse если буду использовать
pub fn cookie_remove_error_response<T: ToString>(body: T, cookie_name: &str) -> Response<axum::body::Body>
{
    
    let error  = body.to_string();
    let body = axum::body::Body::new(error);
    let cookie: Cookie = Cookie::build((cookie_name, "")).path("/").max_age(CookieMaxLife::seconds(0)).into();
    let mut headers = HeaderMap::new();
    headers.insert(
        axum::http::header::SET_COOKIE,
        cookie.to_string().parse().unwrap(),
    );
    let mut resp = Response::builder()
    .status(StatusCode::UNAUTHORIZED)
    .body(body)
    .unwrap();
    *resp.headers_mut() = headers;
    resp
}