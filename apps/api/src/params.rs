use primitives::Chain;
use rocket::request::FromParam;
use std::str::FromStr;

pub struct ChainParam(pub Chain);

impl<'r> FromParam<'r> for ChainParam {
    type Error = &'r str;

    fn from_param(param: &'r str) -> Result<Self, Self::Error> {
        Chain::from_str(param).map(ChainParam).map_err(|_| param)
    }
}

impl From<ChainParam> for Chain {
    fn from(param: ChainParam) -> Self {
        param.0
    }
}

impl AsRef<Chain> for ChainParam {
    fn as_ref(&self) -> &Chain {
        &self.0
    }
}

impl std::ops::Deref for ChainParam {
    type Target = Chain;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
