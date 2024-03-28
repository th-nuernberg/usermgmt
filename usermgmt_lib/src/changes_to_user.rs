use crate::{prelude::*, Entity};
use derive_more::{AsRef, Deref};
#[derive(Debug, AsRef, Deref)]
pub struct ChangesToUser(Entity);

impl ChangesToUser {
    /// # Error
    ///
    /// - if qos and default qos have to be provided together or neither of them.
    pub fn try_new(entity: Entity) -> AppResult<Self> {
        match (&entity.qos, &entity.default_qos) {
            (Some(_), Some(_)) => Ok(Self(entity)),
            (None, None) => Ok(Self(entity)),
            _ => Err(anyhow!(
                "Qos and default Qos must be provided and changed together."
            )),
        }
    }

    /// # Returns Some
    ///
    /// Only if `qos` and `default qos` are to be changed together.
    pub fn may_qos_and_default_qos(&self) -> Option<(Vec<String>, String)> {
        let entity = &self.0;
        match (&entity.qos, &entity.default_qos) {
            (Some(qos), Some(default_qos)) => Some((qos.clone().into(), default_qos.to_string())),
            (None, None) => None,
            _ => unreachable!(),
        }
    }
}
