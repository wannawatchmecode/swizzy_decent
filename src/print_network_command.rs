
use log::{info};
use crate::cli::{CliCommandContext, ExecutableCommand};


use crate::network::NETWORK_DETAILS_STORE;


pub struct PrintNetworkCommandArgs {
}

pub struct PrintNetworkCommand {
    name: String,
    args: PrintNetworkCommandArgs
}


impl ExecutableCommand for PrintNetworkCommand {
    fn execute(&self, _context: CliCommandContext) {
        info!("{:?}", NETWORK_DETAILS_STORE);
    }
}

impl TryFrom<String> for PrintNetworkCommand {
    type Error = ();

    fn try_from(_value: String) -> Result<Self,()> {
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
