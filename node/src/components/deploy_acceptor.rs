mod config;
mod event;

use std::{collections::HashMap, convert::Infallible, fmt::Debug};

use semver::Version;
use serde::Serialize;
use thiserror::Error;
use tracing::{debug, error, info};

use super::chainspec_loader::DeployConfig;
use crate::{
    components::{chainspec_loader::Chainspec, Component},
    effect::{
        announcements::DeployAcceptorAnnouncement,
        requests::{ContractRuntimeRequest, StorageRequest},
        EffectBuilder, EffectExt, Effects,
    },
    types::{Deploy, NodeId},
    utils::Source,
    NodeRng,
};
use casper_types::Key;

use crate::effect::Responder;
pub use config::Config;
pub use event::Event;

#[derive(Debug, Error)]
pub enum Error {
    /// An invalid deploy was received from the client.
    #[error("invalid deploy")]
    InvalidDeploy,
    /// An invalid account sent a deploy
    #[error("invalid account")]
    InvalidAccount,
}

/// A helper trait constraining `DeployAcceptor` compatible reactor events.
pub trait ReactorEventT:
    From<Event>
    + From<DeployAcceptorAnnouncement<NodeId>>
    + From<StorageRequest>
    + From<ContractRuntimeRequest>
    + Send
{
}

impl<REv> ReactorEventT for REv where
    REv: From<Event>
        + From<DeployAcceptorAnnouncement<NodeId>>
        + From<StorageRequest>
        + From<ContractRuntimeRequest>
        + Send
{
}

#[derive(Debug, Clone, Serialize)]
pub struct DeployAcceptorChainspec {
    chain_name: String,
    deploy_config: DeployConfig,
}

impl From<Chainspec> for DeployAcceptorChainspec {
    fn from(c: Chainspec) -> Self {
        DeployAcceptorChainspec {
            chain_name: c.genesis.name,
            deploy_config: c.genesis.deploy_config,
        }
    }
}

/// The `DeployAcceptor` is the component which handles all new `Deploy`s immediately after they're
/// received by this node, regardless of whether they were provided by a peer or a client.
///
/// It validates a new `Deploy` as far as possible, stores it if valid, then announces the newly-
/// accepted `Deploy`.
#[derive(Debug)]
pub struct DeployAcceptor {
    cached_deploy_configs: HashMap<Version, DeployAcceptorChainspec>,
    verify_accounts: bool,
}

impl DeployAcceptor {
    pub(crate) fn new(config: Config) -> Self {
        DeployAcceptor {
            cached_deploy_configs: HashMap::new(),
            verify_accounts: config.verify_accounts(),
        }
    }

    /// Handles receiving a new `Deploy` from a peer or client.
    /// In the case of a peer, there should be no responder and the variant should be `None`
    /// In the case of a client, there should be a responder to communicate the validity of the
    /// deploy and the variant will be `Some`
    fn accept<REv: ReactorEventT>(
        &mut self,
        effect_builder: EffectBuilder<REv>,
        deploy: Box<Deploy>,
        source: Source<NodeId>,
        responder: Option<Responder<Result<(), Error>>>,
    ) -> Effects<Event> {
        // TODO - where to get version from?
        let chainspec_version = Version::new(1, 0, 0);
        let cached_config = self.cached_deploy_configs.get(&chainspec_version).cloned();
        match cached_config {
            Some(genesis_config) => {
                effect_builder
                    .immediately()
                    .event(move |_| Event::GetChainspecResult {
                        deploy,
                        source,
                        chainspec_version,
                        maybe_chainspec: Box::new(Some(genesis_config)),
                        maybe_responder: responder,
                    })
            }
            None => effect_builder
                .get_chainspec(chainspec_version.clone())
                .event(move |maybe_chainspec| Event::GetChainspecResult {
                    deploy,
                    source,
                    chainspec_version,
                    maybe_chainspec: Box::new(maybe_chainspec.map(|c| (*c).clone().into())),
                    maybe_responder: responder,
                }),
        }
    }

    fn account_verification<REv: ReactorEventT>(
        &mut self,
        effect_builder: EffectBuilder<REv>,
        deploy: Box<Deploy>,
        source: Source<NodeId>,
        account_key: Key,
        verified: bool,
        maybe_responder: Option<Responder<Result<(), Error>>>,
    ) -> Effects<Event> {
        let mut effects = Effects::new();
        if !verified {
            info! {
                "Received deploy from invalid account using {}", account_key
            };
            // The client has submitted an invalid deploy. Return an error message to the RPC
            // component via the responder.
            if let Some(responder) = maybe_responder {
                effects.extend(responder.respond(Err(Error::InvalidAccount)).ignore());
            }
            effects.extend(
                effect_builder
                    .announce_invalid_deploy(deploy, source)
                    .ignore(),
            );
            return effects;
        }

        // The client submitted a valid deploy. Return an Ok status to the RPC component via the
        // responder.
        if let Some(responder) = maybe_responder {
            effects.extend(responder.respond(Ok(())).ignore());
        }

        effects.extend(
            effect_builder
                .put_deploy_to_storage(deploy.clone())
                .event(move |is_new| Event::PutToStorageResult {
                    deploy,
                    source,
                    is_new,
                }),
        );

        effects
    }

    fn validate<REv: ReactorEventT>(
        &mut self,
        effect_builder: EffectBuilder<REv>,
        deploy: Box<Deploy>,
        source: Source<NodeId>,
        chainspec: DeployAcceptorChainspec,
        maybe_responder: Option<Responder<Result<(), Error>>>,
    ) -> Effects<Event> {
        let mut cloned_deploy = deploy.clone();
        let mut effects = Effects::new();
        let is_acceptable =
            cloned_deploy.is_acceptable(chainspec.chain_name, chainspec.deploy_config);
        if !is_acceptable {
            // The client has submitted an invalid deploy. Return an error message to the RPC
            // component via the responder.
            if let Some(responder) = maybe_responder {
                effects.extend(responder.respond(Err(Error::InvalidDeploy)).ignore());
            }
            effects.extend(
                effect_builder
                    .announce_invalid_deploy(deploy, source)
                    .ignore(),
            );
            return effects;
        }

        let account_key = deploy.header().account().to_owned().into();

        // skip account verification if deploy not received from client or node is configured to
        // not verify accounts
        if !source.from_client() || !self.verify_accounts {
            return effect_builder
                .immediately()
                .event(move |_| Event::AccountVerificationResult {
                    deploy,
                    source,
                    account_key,
                    verified: true,
                    maybe_responder,
                });
        }

        effect_builder
            .is_verified_account(account_key)
            .event(move |verified| Event::AccountVerificationResult {
                deploy,
                source,
                account_key,
                verified,
                maybe_responder,
            })
    }

    fn failed_to_get_chainspec(
        &self,
        deploy: Box<Deploy>,
        source: Source<NodeId>,
        chainspec_version: Version,
    ) -> Effects<Event> {
        error!(%deploy, %source, %chainspec_version, "failed to get chainspec");
        Effects::new()
    }

    fn handle_put_to_storage<REv: ReactorEventT>(
        &mut self,
        effect_builder: EffectBuilder<REv>,
        deploy: Box<Deploy>,
        source: Source<NodeId>,
        is_new: bool,
    ) -> Effects<Event> {
        if is_new {
            return effect_builder
                .announce_new_deploy_accepted(deploy, source)
                .ignore();
        }
        Effects::new()
    }
}

impl<REv: ReactorEventT> Component<REv> for DeployAcceptor {
    type Event = Event;
    type ConstructionError = Infallible;

    fn handle_event(
        &mut self,
        effect_builder: EffectBuilder<REv>,
        _rng: &mut NodeRng,
        event: Self::Event,
    ) -> Effects<Self::Event> {
        debug!(?event, "handling event");
        match event {
            Event::Accept {
                deploy,
                source,
                responder,
            } => self.accept(effect_builder, deploy, source, responder),
            Event::GetChainspecResult {
                deploy,
                source,
                chainspec_version,
                maybe_chainspec,
                maybe_responder,
            } => match *maybe_chainspec {
                Some(chainspec) => {
                    // Update chainspec cache.
                    self.cached_deploy_configs
                        .insert(chainspec_version, chainspec.clone());
                    self.validate(effect_builder, deploy, source, chainspec, maybe_responder)
                }
                None => self.failed_to_get_chainspec(deploy, source, chainspec_version),
            },
            Event::PutToStorageResult {
                deploy,
                source,
                is_new,
            } => self.handle_put_to_storage(effect_builder, deploy, source, is_new),
            Event::AccountVerificationResult {
                deploy,
                source,
                account_key,
                verified,
                maybe_responder,
            } => self.account_verification(
                effect_builder,
                deploy,
                source,
                account_key,
                verified,
                maybe_responder,
            ),
        }
    }
}
