use cosmwasm_std::{StdResult, Storage};
use cosmwasm_storage::{singleton, singleton_read};
use basset_vault::basset_vault_config_holder::Config;

static KEY_CONFIG: &[u8] = b"config";

pub fn load_config(storage: &dyn Storage) -> StdResult<Config> {
    singleton_read(storage, KEY_CONFIG).load()
}

pub fn save_config(storage: &mut dyn Storage, config: &Config) -> StdResult<()> {
    singleton(storage, KEY_CONFIG).save(config)
}
