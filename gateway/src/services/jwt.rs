use std::sync::{Arc};
use jwt_authentification::{Claims, JWT};
use tracing::error;

use crate::{error::Error};

pub struct JWTService
{
    jwt: Arc<JWT>
}

impl JWTService
{
    pub fn new() -> Result<Self, Error>
    {
        Ok(Self
        {
            jwt: Arc::new(JWT::new_in_file("key.pkcs8")?),
        })
        
    }
    ///Генерирование нового access ключа
    /// `lifetime` - время жизни ключа в минутах
    pub fn gen_key<T: ToString>(&self, id: T, lifetime: i64) -> Result<String, Error> 
    {
        let key = self.jwt.generate_key(id, lifetime)
            .inspect_err(|e| error!("{}", e))?;
        Ok(key)
    }
    ///validate access key, validation will not be performed on roles and audience if they are empty
    pub async fn validate_id<T>(&self, user_id: T, token: &str) -> Result<Claims, Error>
    where 
        T: ToString
    {
        let data = self.jwt.validator()
            .with_subject(user_id)
            .validate(token)?;
        Ok(data.claims)
    }
    pub async fn validate(&self, token: &str) -> Result<Claims, Error>
    {
        let data = self.jwt.validator()
            .validate(token)?;
        Ok(data.claims)
    }
}