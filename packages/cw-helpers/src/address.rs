use cosmwasm_std::{Addr, Api, CanonicalAddr, StdResult};
pub enum Address<'a> {
    Canonical(CanonicalAddr, &'a dyn Api),
    Addr(Addr, &'a dyn Api),
    Str(String, &'a dyn Api),
}

impl Address<'_> {
    pub fn to_canonical(&self) -> StdResult<CanonicalAddr> {
        match self {
            Address::Canonical(addr, _) => Ok(addr.clone()),
            Address::Addr(addr, api) => Ok(api.addr_canonicalize(&addr.to_string())?),
            Address::Str(addr, api) => Ok(api.addr_canonicalize(addr)?),
        }
    }

    pub fn to_addr(&self) -> StdResult<Addr> {
        match self {
            Address::Canonical(addr, api) => Ok(api.addr_humanize(addr)?),
            Address::Addr(addr, _) => Ok(addr.clone()),
            Address::Str(addr, api) => Ok(api.addr_validate(addr)?),
        }
    }

    pub fn to_string(&self) -> StdResult<String> {
        match self {
            Address::Canonical(addr, api) => Ok(api.addr_humanize(addr)?.to_string()),
            Address::Addr(addr, _) => Ok(addr.to_string()),
            Address::Str(addr, _) => Ok(addr.clone()),
        }
    }
}
