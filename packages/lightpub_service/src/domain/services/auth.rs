use derive_more::Constructor;

use crate::{
    ServiceResult,
    domain::models::user::{HashedUserPassword, UserAuthEntity, UserID, UserPassword},
};

pub trait PasswordHashService {
    fn hash_password(&self, password: &UserPassword) -> ServiceResult<HashedUserPassword>;

    fn create_user_auth(
        &self,
        user_id: UserID,
        password: &UserPassword,
    ) -> ServiceResult<UserAuthEntity> {
        let hashed_password = self.hash_password(password)?;
        Ok(UserAuthEntity::new(user_id, hashed_password))
    }

    fn update_user_auth(
        &self,
        user_auth: &mut UserAuthEntity,
        password: &UserPassword,
    ) -> ServiceResult<()> {
        let hashed_password = self.hash_password(password)?;
        user_auth.set_password_hash(hashed_password);
        Ok(())
    }
}

#[derive(Debug, Constructor)]
pub struct BcryptHashService;

impl PasswordHashService for BcryptHashService {
    fn hash_password(&self, password: &UserPassword) -> ServiceResult<HashedUserPassword> {
        let res = bcrypt::hash(&password, bcrypt::DEFAULT_COST).map_err_unknown()?;
        Ok(res)
    }
}
