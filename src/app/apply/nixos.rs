//! `service:` and `firewall:` on a NixOS machine — `J5`'s fourth ruling, which was *everything*.
//!
//! **On NixOS these are not commands, they are configuration.** `systemctl enable nginx` writes
//! into a tree the next `nixos-rebuild switch` regenerates, so an enablement Shall issued
//! imperatively survives exactly until the next generation — including the generation Shall
//! itself builds one line later when a `nixos:` package changes. And `ufw` is not there at all:
//! a NixOS box declaring `firewall:22/tcp` reached [`Firewall::apply`](super::Firewall::apply),
//! found no adapter, and failed the whole sync by name. So the perimeter and the services go
//! into the same generated module the packages do, and one `nixos-rebuild` applies all three.
//!
//! **What is not moved here: a restart.** `services.<name>.enable` is a state; `@status=
//! restarted` is a transition, and no attribute in a NixOS module can express it. Those lines
//! keep going through the init exactly as on every other host — see
//! [`nixos::service_routing`](crate::backends::nixos::service_routing) for the split and
//! [`imperative_remainder`] for what `Dependents` is left holding.
//!
//! **The safety story is not weakened by the move.** The lockout check, the removal guard and
//! the addition ceiling all run here too, against the same functions the adapter path calls —
//! because a port dropped from `allowedTCPPorts` closes just as hard as one `ufw delete` takes
//! away, and it closes on a machine whose rebuild takes minutes to undo.

use crate::config::grammar::{Options, Statement};
use crate::core::{Error, Result};
use crate::model::firewall::{Direction, Proto, Rule};
use std::collections::{BTreeMap, BTreeSet};
use tracing::info;

/// Holds exactly what it uses, like every other applier in this module.
pub struct SystemConfig<'a> {
    pub(crate) config: &'a std::sync::Arc<crate::config::Config>,
    pub(crate) executor: &'a crate::core::CommandExecutor,
    pub(crate) registry: &'a std::sync::Arc<crate::backends::BackendRegistry>,
    pub(crate) reaping: &'a crate::app::sync::guard::Reaping,
}

/// What a desired state says the system configuration should hold, before anything on disk is
/// consulted. Pure, so every rule below is testable without a NixOS.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Declared {
    pub services: BTreeMap<String, bool>,
    pub tcp_ports: BTreeSet<u16>,
    pub udp_ports: BTreeSet<u16>,
    pub firewall: Option<bool>,
    /// Whether any `service:` or `firewall:` line was declared at all. Not the same as the
    /// fields being empty: a config that used to declare a service and no longer does has an
    /// empty map and real work to do.
    pub any: bool,
}

impl Declared {
    fn ports(&self) -> impl Iterator<Item = (u16, Proto)> + '_ {
        self.tcp_ports
            .iter()
            .map(|p| (*p, Proto::Tcp))
            .chain(self.udp_ports.iter().map(|p| (*p, Proto::Udp)))
    }

    fn rules(&self) -> Vec<Rule> {
        self.ports()
            .map(|(port, proto)| Rule::Port { port, proto })
            .collect()
    }
}

/// Project a desired state onto the attributes NixOS reads.
///
/// Every refusal in here is P7's rule: a line this OS cannot express is named, never
/// approximated. A half-applied perimeter or a service that reports enabled and is not are both
/// worse than a message telling the user which line to change.
pub fn declared(state: &crate::model::DesiredState) -> Result<Declared> {
    let mut out = Declared::default();

    for (stmt, origin) in state.dependents() {
        let Statement::Service(name, opts) = stmt else {
            continue;
        };
        out.any = true;
        if opts.one("user").is_some() {
            continue;
        }
        let (enable, _restart) =
            crate::backends::nixos::service_routing(opts.one("enabled"), opts.one("status"))
                .map_err(|e| Error::Validation(format!("{}: `service:{}` {}", origin, name, e)))?;
        if let Some(on) = enable {
            // A name declared twice with two answers is the same contradiction one line apart,
            // and it must not be settled by whichever module happened to load last.
            if let Some(previous) = out.services.insert(name.clone(), on) {
                if previous != on {
                    return Err(Error::Validation(format!(
                        "{}: `service:{}` is declared both on and off. On NixOS that is one \
                         attribute, `services.{}.enable`, so the two lines cannot both hold.",
                        origin, name, name
                    )));
                }
            }
        }
    }

    for (name, opts, origin) in state.firewall_rules() {
        out.any = true;
        let rule =
            Rule::parse(name).map_err(|e| Error::Validation(format!("{}: {}", origin, e)))?;
        match rule {
            Rule::Port { port, proto } => {
                match proto {
                    Proto::Tcp => out.tcp_ports.insert(port),
                    Proto::Udp => out.udp_ports.insert(port),
                };
            }
            Rule::Default {
                direction: Direction::Incoming,
            } => {
                let policy = opts.one("value").map(str::trim).unwrap_or("deny");
                out.firewall = Some(match policy {
                    "deny" | "reject" | "drop" => true,
                    "allow" | "accept" => false,
                    other => {
                        return Err(Error::Validation(format!(
                            "{}: `firewall:default/incoming @value={}` names no policy NixOS \
                             has. It is `deny` (which is `networking.firewall.enable = true`) \
                             or `allow` (which turns the firewall off).",
                            origin, other
                        )))
                    }
                });
            }
            // NixOS's `networking.firewall` filters what comes in. There is no attribute for a
            // default outgoing policy, and inventing one out of raw nftables rules would be
            // Shall writing a firewall rather than declaring one.
            Rule::Default {
                direction: Direction::Outgoing,
            } => {
                return Err(Error::Validation(format!(
                    "{}: `firewall:default/outgoing` asks for a default outgoing policy, and \
                     NixOS's `networking.firewall` has no such option — it filters incoming \
                     traffic. Remove the line, or write the rule yourself with \
                     `networking.firewall.extraCommands` in your own configuration.",
                    origin
                )))
            }
        }
    }

    Ok(out)
}

/// What `Dependents` is still left to do with a `service:` line here: `None` when the generated
/// module says all of it.
///
/// Returns the options trimmed to the transition alone, so the init is asked for the restart and
/// not for an enablement the configuration already owns. Passing the original options through
/// would re-issue `systemctl enable` beside `services.<name>.enable`, which is the two-owners
/// problem this whole module exists to remove.
pub fn imperative_remainder(opts: &Options) -> Option<Options> {
    let (_, restart) =
        crate::backends::nixos::service_routing(opts.one("enabled"), opts.one("status")).ok()?;
    if !restart {
        return None;
    }
    let mut trimmed = Options::default();
    trimmed.set("status", "restarted");
    Some(trimmed)
}

/// Whether this machine's `service:` and `firewall:` lines are the system configuration's
/// business.
///
/// Asked of the registered backend rather than re-testing `/etc/NIXOS`, so there is one answer to
/// *is this NixOS*. A free function because two callers need it and only one of them is a
/// [`SystemConfig`]: [`Dependents`](super::Dependents) has to know to pass the `service:` lines
/// over, and a second copy of this test in that file is how the two would come to disagree.
pub fn owns_extras(registry: &crate::backends::BackendRegistry) -> bool {
    registry
        .get("nixos")
        .map(|b| b.is_available())
        .unwrap_or(false)
}

impl SystemConfig<'_> {
    pub fn owns_extras(&self) -> bool {
        owns_extras(self.registry)
    }

    fn core(&self) -> crate::backends::nixos::NixosBackendCore {
        crate::backends::nixos::NixosBackendCore::new(
            self.executor.clone(),
            self.config.nixos.config_dir.clone(),
            self.config.nixos.manage_imports,
        )
    }

    /// Write the declared services and perimeter into the system configuration and switch to it.
    pub async fn apply(
        &self,
        state: &crate::model::DesiredState,
        scope: crate::app::sync::guard::GuardScope,
    ) -> Result<()> {
        let want = declared(state)?;
        let core = self.core();
        let current = core.declared();

        // **Nothing declared and nothing already ours is not a reason to write a file.** A NixOS
        // user who only ever says `nix:` should not find `/etc/nixos` edited and an `imports`
        // line added because Shall ran.
        if !want.any && !core.generated_path().exists() {
            return Ok(());
        }

        let wanted_ports: BTreeSet<u16> = want.ports().map(|(p, _)| p).collect();
        let closing: Vec<Rule> = current
            .ports()
            .filter(|(p, _)| !wanted_ports.contains(p))
            .map(|(port, proto)| Rule::Port { port, proto })
            .collect();
        let opening: Vec<Rule> = want
            .rules()
            .into_iter()
            .filter(|r| match r {
                Rule::Port { port, proto } => match proto {
                    Proto::Tcp => !current.tcp_ports.contains(port),
                    Proto::Udp => !current.udp_ports.contains(port),
                },
                Rule::Default { .. } => false,
            })
            .collect();
        let disabling: Vec<(String, String)> = current
            .services
            .keys()
            .filter(|name| !want.services.contains_key(*name))
            .map(|name| ("service".to_string(), name.clone()))
            .collect();

        // THE PRECONDITION, before anything is written. A port that leaves `allowedTCPPorts`
        // is shut by the rebuild just as surely as by `ufw delete`, and the machine that shuts
        // it takes minutes to rebuild back.
        if let Some(port) = crate::model::firewall::would_close_session(
            &closing,
            want.firewall == Some(true),
            &want.rules(),
            crate::model::firewall::session_port(),
        ) {
            return Err(Error::Refused(crate::model::firewall::lockout_refusal(
                port, scope,
            )));
        }

        if self.config.dry_run {
            for r in &opening {
                crate::would!("would open {} in the NixOS system configuration", r);
            }
            for r in &closing {
                crate::would!("would close {} in the NixOS system configuration", r);
            }
            for (name, on) in &want.services {
                if current.services.get(name) != Some(on) {
                    crate::would!("would declare services.{}.enable = {}", name, on);
                }
            }
            for (_, name) in &disabling {
                crate::would!("would stop declaring services.{}", name);
            }
            return Ok(());
        }

        // Placing takes nothing away; it is still a change, and `max_total_changes` counts
        // changes (`N8`).
        let additions = opening.len()
            + want
                .services
                .iter()
                .filter(|(name, on)| current.services.get(*name) != Some(on))
                .count();
        crate::app::sync::guard::enforce_additions(self.config, additions, self.reaping, scope)
            .await?;

        // Two budgets, because they are two kinds of loss: a closed port answers to
        // `max_port_closures`, an undeclared resource to `max_extra_removals` (`N8`).
        if !closing.is_empty() {
            let removals: Vec<(String, String)> = closing
                .iter()
                .map(|r| ("firewall".to_string(), r.to_string()))
                .collect();
            let _reaped = crate::app::sync::guard::enforce_ports(
                self.config,
                self.registry,
                &removals,
                self.reaping,
                scope,
            )
            .await?;
        }
        if !disabling.is_empty() {
            let _reaped = crate::app::sync::guard::enforce_extras(
                self.config,
                self.registry,
                &disabling,
                self.reaping,
                scope,
            )
            .await?;
        }

        let module = crate::backends::nixos::Module {
            // The packages are the other writer's half and are carried through untouched.
            packages: current.packages,
            services: want.services.clone(),
            tcp_ports: want.tcp_ports.clone(),
            udp_ports: want.udp_ports.clone(),
            firewall: want.firewall,
            config_text: current.config_text,
        };
        core.write_and_switch(&module).await?;
        for r in &opening {
            info!("opened {} (NixOS system configuration)", r);
        }
        for r in &closing {
            info!("closed {} — it was not declared (NixOS)", r);
        }
        Ok(())
    }
}
