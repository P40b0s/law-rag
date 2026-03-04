use serde::{Deserialize, Serialize};
use crate::services::{UserDbo, AccessDbo};
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User
{
    pub id: uuid::Uuid,
    pub username: String,
    pub password: String,
    pub first_name: String,
    pub second_name: String,
    pub surname: String,
    pub avatar: Option<Vec<u8>>,
    pub access: Vec<Access>
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Access
{
    pub service_name: String,
    pub permissions: Vec<String>,
}

impl From<AccessDbo> for Access
{
    fn from(value: AccessDbo) -> Self 
    {
        Access 
        { 
            service_name: value.service_name,
            permissions: value.permissions
        }
    }
}


impl From<UserDbo> for User
{
    fn from(value: UserDbo) -> Self 
    {
        User 
        { 
            id: value.id,
            username: value.username,
            password: value.password,
            first_name: value.first_name,
            second_name: value.second_name,
            surname: value.surname,
            avatar: value.avatar,
            access: value.access.into_iter().map(Into::into).collect()
        }
    }
}