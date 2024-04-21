use std::io::{BufRead, stdin};
use std::net::{AddrParseError, IpAddr, SocketAddr};
use std::str::FromStr;
use std::sync::mpsc::Sender;
use log::error;
use crate::health_check::{HEALTH_CHECK_SYN_OPCODE, HealthCheckPacket};
use crate::health_check_network_broker::{HealthCheckNetworkBrokerMessage};
use crate::print_network_command::PrintNetworkCommand;
use crate::utils::generate_nonce;

pub struct SwizzyDecentCli {
    request_sender: Sender<HealthCheckNetworkBrokerMessage>
}

pub struct SwizzyDecentCliRunConfiguration {

}

struct CliCommand<A> {
    pub name: String,
    pub args: A
}

struct HealthCheckCommandArgs {
    socket_addr: SocketAddr,
}

struct HealthCheckCommand {
    name: String,
    args: HealthCheckCommandArgs
}

pub(crate) trait ExecutableCommand {
    fn execute(&self, context: CliCommandContext);
}

pub(crate) struct CliCommandContext {
    pub request_sender: Sender<HealthCheckNetworkBrokerMessage>,
}

struct HealthCheckCommandResult {

}

impl ExecutableCommand for HealthCheckCommand {
    fn execute(&self, context: CliCommandContext) {
        let res = context.request_sender.send(HealthCheckNetworkBrokerMessage {
            payload: HealthCheckPacket {
                header: HEALTH_CHECK_SYN_OPCODE,
                nonce: generate_nonce()
            },
            remote_addr: self.args.socket_addr,
        });
        if res.is_err() {
            error!("Error executing health check command");
        }
    }
}

impl TryFrom<String> for HealthCheckCommand {
    type Error = ();

    fn try_from(value: String) -> Result<Self,()> {
        let args = value.split(" ");

        let mut is_first = true;
        let mut remote_addr:Result<IpAddr, AddrParseError> = IpAddr::from_str("asd");
        let mut port: Option<u16> = Option::None;
        for arg in args {
            if is_first {
                if arg != "hc" {
                    return Err(());
                }
                is_first = false;
                continue;
            }

            let mut key_val_itr = arg.split("=");
            match key_val_itr.next().unwrap() {
                "--remote_addr" => remote_addr = IpAddr::from_str(key_val_itr.next().unwrap()),
                "--port" => port = Option::from(String::from(key_val_itr.next().unwrap()).parse::<u16>().unwrap()),
                _ => ()
            }
        }

        if remote_addr.is_err() || port.is_none() {
            return Err(());
        }

        let socket_addr = SocketAddr::new(remote_addr.unwrap(), port.unwrap());

        return Ok(HealthCheckCommand {
            name: String::from("hc"),
            args: HealthCheckCommandArgs {
                socket_addr
            }
        });
    }
}

impl Into<CliCommand<HealthCheckCommandArgs>> for HealthCheckCommand {
    fn into(self) -> CliCommand<HealthCheckCommandArgs> {
        return CliCommand {
            name: self.name,
            args: self.args
        };
    }
}

impl SwizzyDecentCli {

    pub fn new(request_sender: Sender<HealthCheckNetworkBrokerMessage>) -> Self {
        return SwizzyDecentCli {
            request_sender
        }
    }
    pub fn run(self, _configuration: SwizzyDecentCliRunConfiguration) {
        let request_sender = self.request_sender.clone();
        loop {
            let command_res = get_command();
            if command_res.is_err() {
                error!("Error parsing command, please try again");
                continue;
            }

            let command = command_res.unwrap();
            let context = CliCommandContext {
                request_sender: request_sender.clone()
            };
            command.execute(context)
        }
    }
}

fn get_command() -> Result<Box<dyn ExecutableCommand>, ()> {
    // TODO: Probably should read one line at a time.
    for line in stdin().lock().lines() {
        let line = line.unwrap();
        let name = get_name(line.clone());
        return parse_command_by_name(name, line)
    }

    return Err(())


}

fn parse_command_by_name(name: String, line: String) -> Result<Box<dyn ExecutableCommand>, ()>  {

    match name.as_str() {
        "hc" => {
            let command = HealthCheckCommand::try_from(line);
            if command.is_err() {
                return Err(())
            }
            return Ok(Box::new(command.unwrap()));
        },
        "pn" => {
            let command = PrintNetworkCommand::try_from(line);
            if command.is_err() {
                return Err(())
            }
            return Ok(Box::new(command.unwrap()));
        },
                      _ => return Err(())
    }
}

fn get_name(line: String) -> String {
    let arg = line.split(" ").next().unwrap();
    return String::from(arg)
}