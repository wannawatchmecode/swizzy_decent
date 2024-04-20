use std::io::{BufRead, stdin};
use std::io::ErrorKind::{AddrInUse, AddrNotAvailable};
use std::net::{AddrParseError, IpAddr, SocketAddr};
use std::str::FromStr;
use std::sync::mpsc::Sender;
use std::time::{SystemTime, UNIX_EPOCH};
use crate::health_check::{HEALTH_CHECK_SYN_OPCODE, HealthCheckPacket};
use crate::health_check_network_broker::{HealthCheckNetworkBroker, HealthCheckNetworkBrokerMessage};
use crate::HealthCheckCli;
use crate::network::NetworkDetailsStore;
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

// struct CliCommandResult<A> {
//
// }

struct HealthCheckCommandArgs {
    socket_addr: SocketAddr,
}

// const HEALTH_CHECK_COMMAND_SHORT_KEY: String = String::from("hc");

struct HealthCheckCommand {
    name: String,
    args: HealthCheckCommandArgs
}

trait ExecutableCommand {
    fn execute(&self, context: CliCommandContext);
}

struct CliCommandContext {
// <'a> {
    // network_details_store: &'a NetworkDetailsStore,
    pub request_sender: Sender<HealthCheckNetworkBrokerMessage>
}

struct HealthCheckCommandResult {

}

impl ExecutableCommand for HealthCheckCommand {
    fn execute(&self, context: CliCommandContext) {
        context.request_sender.send(HealthCheckNetworkBrokerMessage {
            payload: HealthCheckPacket {
                header: HEALTH_CHECK_SYN_OPCODE,
                nonce: generate_nonce()
            },
            remote_addr: self.args.socket_addr,
        }).expect("Message to be sent")
        // return HealthCheckCommandResult { }
    }
}

impl TryFrom<String> for HealthCheckCommand {
    type Error = ();

    fn try_from(value: String) -> Result<Self,()> {
        let args = value.split(" ");
        // if args[0] != HEALTH_CHECK_COMMAND_SHORT_KEY {
        //     return Err(());
        // }

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
        // return String::from(value[0])
        todo!()
    }
}

// impl From<ExecutableCommand> for HealthCheckCommand {
//     fn from(value: CliCommand<HealthCheckCommandArgs>) -> Self {
//         todo!()
//     }
// }

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
    pub fn run(self, configuration: SwizzyDecentCliRunConfiguration) {
        let request_sender = self.request_sender.clone();
        loop {
            let command = get_command().unwrap();
            let context = CliCommandContext {
                request_sender: request_sender.clone()
            };
            command.execute(context)
        }
    }


    // fn parse_command<A>(line: &str) -> CliCommand<A> {
    //
    // }
}

fn get_command() -> Result<Box<dyn ExecutableCommand>, ()> {
    // TODO: Probably should read one line at a time.
    // let mut result: Result<Box<dyn ExecutableCommand>, ()> = Result::Err(());
    for line in stdin().lock().lines() {
        let line = line.unwrap();
        let name = get_name(line.clone());
        return Ok(parse_command_by_name(name, line).expect("Should convert line to health check command"))
    }

    return Err(())


}

fn parse_command_by_name(name: String, line: String) -> Result<Box<dyn ExecutableCommand>, ()>  {

    match name.as_str() {
        "hc" => return Ok(Box::new(HealthCheckCommand::try_from(line)
            .expect("Should convert line to health check command"))),
        _ => return Err(())
    }
}

fn get_name(line: String) -> String {
    let arg = line.split(" ").next().unwrap();
    return String::from(arg)
}