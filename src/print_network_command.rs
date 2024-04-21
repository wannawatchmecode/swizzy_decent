use std::net::{AddrParseError, IpAddr, SocketAddr};
use log::{error, info};
use crate::cli::{CliCommandContext, ExecutableCommand};
use crate::health_check::{HEALTH_CHECK_SYN_OPCODE, HealthCheckPacket};
use crate::health_check_network_broker::HealthCheckNetworkBrokerMessage;
use crate::network::NETWORK_DETAILS_STORE;
use crate::utils::generate_nonce;

pub struct PrintNetworkCommandArgs {
}

pub struct PrintNetworkCommand {
    name: String,
    args: PrintNetworkCommandArgs
}


impl ExecutableCommand for PrintNetworkCommand {
    fn execute(&self, context: CliCommandContext) {
        info!("{:?}", NETWORK_DETAILS_STORE);
    }
}

impl TryFrom<String> for PrintNetworkCommand {
    type Error = ();

    fn try_from(value: String) -> Result<Self,()> {
        return Ok(PrintNetworkCommand {
            name: String::from("pn"),
            args: PrintNetworkCommandArgs {
            }
        });
    }
}

// impl Into<crate::cli::CliCommand<HealthCheckCommandArgs>> for HealthCheckCommand {
//     fn into(self) -> crate::cli::CliCommand<HealthCheckCommandArgs> {
//         return crate::cli::CliCommand {
//             name: self.name,
//             args: self.args
//         };
//     }
// }
