use core_chain_traits::{ChainAccount, ChainAddressStatus, ChainPerpetual, ChainProvider, ChainTraits};
use core_client::Client;
use primitives::Chain;

use crate::rpc::client::EthereumClient;

impl<C: Client + Clone> ChainTraits for EthereumClient<C> {}

impl<C: Client + Clone> ChainProvider for EthereumClient<C> {
    fn get_chain(&self) -> Chain {
        self.get_chain()
    }
}

impl<C: Client + Clone> ChainAccount for EthereumClient<C> {}

impl<C: Client + Clone> ChainPerpetual for EthereumClient<C> {}

impl<C: Client + Clone> ChainAddressStatus for EthereumClient<C> {}
