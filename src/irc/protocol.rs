#![cfg_attr(feature = "clippy", allow(block_in_if_condition_stmt))]  // impl_rdp! uses this



use slog::{Record, Serialize, Serializer};
use slog::ser::Error as SlogSerError;

use std::convert::From;
use std::str;
use std::str::FromStr;


#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum IrcCommand {
    Nick { nick: String },
    User { user: String, real_name: String },
    Join { channel: String },
    Part { channel: String },
    Quit,
    Ping { data: String },
    Mode { target: String, mask: Option<String> },
    Pong { data: String },
    Pass { password: String },
    PrivMsg { channel: String, text: String },
    Topic { channel: String, topic: String },
    Who { matches: String },
}

impl IrcCommand {
    pub fn from_irc_line(irc_line: IrcLine) -> Option<IrcCommand> {
        match irc_line.command {
            Command::Nick => {
                irc_line.args.into_iter().next().map(|nick| IrcCommand::Nick { nick: nick })
            }
            Command::User => {
                let mut it = irc_line.args.into_iter();
                if let (Some(user), Some(real_name)) = (it.nth(0), it.nth(2)) {
                    Some(IrcCommand::User {
                        user: user,
                        real_name: real_name,
                    })
                } else {
                    None
                }
            }
            Command::Join => {
                irc_line.args.into_iter().next().map(|arg| IrcCommand::Join { channel: arg })
            }
            Command::Part => {
                irc_line.args.into_iter().next().map(|arg| IrcCommand::Part { channel: arg })
            }
            Command::Quit => Some(IrcCommand::Quit),
            Command::Ping => {
                irc_line.args.into_iter().next().map(|arg| IrcCommand::Ping { data: arg })
            }
            Command::Mode => {
                let mut it = irc_line.args.into_iter();
                if let Some(target) = it.next() {
                    Some(IrcCommand::Mode {
                        target: target,
                        mask: it.next(),
                    })
                } else {
                    None
                }
            }
            Command::Pong => {
                irc_line.args.into_iter().next().map(|arg| IrcCommand::Pong { data: arg })
            }
            Command::Pass => {
                irc_line.args.into_iter().next().map(|arg| IrcCommand::Pass { password: arg })
            }
            Command::PrivMsg => {
                let mut it = irc_line.args.into_iter();
                if let (Some(channel), Some(text)) = (it.nth(0), it.nth(0)) {
                    Some(IrcCommand::PrivMsg {
                        channel: channel,
                        text: text,
                    })
                } else {
                    None
                }
            }
            Command::Topic => {
                let mut it = irc_line.args.into_iter();
                if let (Some(channel), Some(topic)) = (it.nth(0), it.nth(0)) {
                    Some(IrcCommand::Topic {
                        channel: channel,
                        topic: topic,
                    })
                } else {
                    None
                }
            }
            Command::Who => {
                irc_line.args.into_iter().next().map(|arg| IrcCommand::Who { matches: arg })
            }
            Command::Numeric { .. } |
            Command::Unknown => None,
        }
    }

    pub fn command(&self) -> Command {
        match *self {
            IrcCommand::Nick { .. } => Command::Nick,
            IrcCommand::User { .. } => Command::User,
            IrcCommand::Join { .. } => Command::Join,
            IrcCommand::Part { .. } => Command::Part,
            IrcCommand::Quit => Command::Quit,
            IrcCommand::Ping { .. } => Command::Ping,
            IrcCommand::Mode { .. } => Command::Mode,
            IrcCommand::Pong { .. } => Command::Pong,
            IrcCommand::Pass { .. } => Command::Pass,
            IrcCommand::PrivMsg { .. } => Command::PrivMsg,
            IrcCommand::Topic { .. } => Command::Topic,
            IrcCommand::Who { .. } => Command::Who,
        }
    }
}

impl FromStr for IrcCommand {
    type Err = ();
    fn from_str(line: &str) -> Result<IrcCommand, ()> {
        parse_irc_line(line).and_then(IrcCommand::from_irc_line).ok_or(())
    }
}


#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Command {
    Nick,
    User,
    Join,
    Part,
    Quit,
    Ping,
    Mode,
    Pong,
    Pass,
    PrivMsg,
    Topic,
    Who,
    Numeric { code: u16, string: [u8; 3] },
    Unknown,
}

impl<'a> From<&'a str> for Command {
    fn from(cmd: &'a str) -> Command {
        match cmd {
            "NICK" => Command::Nick,
            "USER" => Command::User,
            "JOIN" => Command::Join,
            "PART" => Command::Part,
            "QUIT" => Command::Quit,
            "PING" => Command::Ping,
            "PONG" => Command::Pong,
            "MODE" => Command::Mode,
            "PASS" => Command::Pass,
            "PRIVMSG" => Command::PrivMsg,
            "TOPIC" => Command::Topic,
            "WHO" => Command::Who,
            _ => {
                if cmd.len() == 3 {
                    if let Ok(c) = cmd.parse() {
                        Command::Numeric {
                            code: c,
                            string: [cmd.as_bytes()[0], cmd.as_bytes()[1], cmd.as_bytes()[3]],
                        }
                    } else {
                        Command::Unknown
                    }
                } else {
                    Command::Unknown
                }
            }
        }
    }
}

impl From<String> for Command {
    fn from(cmd: String) -> Command {
        match cmd.as_ref() {
            "NICK" => return Command::Nick,
            "USER" => return Command::User,
            "JOIN" => return Command::Join,
            "PART" => return Command::Part,
            "QUIT" => return Command::Quit,
            "PING" => return Command::Ping,
            "PONG" => return Command::Pong,
            "MODE" => return Command::Mode,
            "PASS" => return Command::Pass,
            "PRIVMSG" => return Command::PrivMsg,
            "TOPIC" => return Command::Topic,
            "WHO" => return Command::Who,
            _ => {}
        }

        if cmd.len() == 3 {
            if let Ok(c) = cmd.parse() {
                Command::Numeric {
                    code: c,
                    string: [cmd.as_bytes()[0], cmd.as_bytes()[1], cmd.as_bytes()[2]],
                }
            } else {
                Command::Unknown
            }
        } else {
            Command::Unknown
        }
    }
}

impl Command {
    pub fn to_str(&self) -> &str {
        match *self {
            Command::Nick => "NICK",
            Command::User => "USER",
            Command::Join => "JOIN",
            Command::Part => "PART",
            Command::Quit => "QUIT",
            Command::Ping => "PING",
            Command::Mode => "MODE",
            Command::Pong => "PONG",
            Command::Pass => "PASS",
            Command::PrivMsg => "PRIVMSG",
            Command::Topic => "TOPIC",
            Command::Who => "WHO",
            Command::Numeric { ref string, .. } => str::from_utf8(string).expect("Numeric code"),
            Command::Unknown => "<UNKNOWN>",
        }
    }
}

impl Serialize for Command {
    fn serialize(
        &self,
        _record: &Record,
        key: &str,
        serializer: &mut Serializer
    ) -> Result<(), SlogSerError> {
        serializer.emit_str(key, self.to_str())
    }
}


use pest::prelude::*;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct IrcLine {
    pub prefix: Option<String>,
    pub command: Command,
    pub args: Vec<String>,
}

impl FromStr for IrcLine {
    type Err = ();
    fn from_str(line: &str) -> Result<IrcLine, ()> {
        parse_irc_line(line).ok_or(())
    }
}

#[derive(Default)]
struct IrcLineBuilder {
    prefix: Option<String>,
    command: Option<String>,
    args: Vec<String>,
}

impl_rdp! {
    grammar! {
        expression = _{ ([":"] ~ prefix ~ space )? ~ command ~ params }
        prefix = { ([":"] | nospcrlfcl)+ }
        command = { (['A'..'Z']+) | (digit ~ digit ~ digit) }
        digit = _{ ['0' .. '9'] }
        space = _{ [" "]+ }
        params = _{ (space ~ param)* ~ ( space ~ [":"] ~ trailing_param )?  }
        nospcrlfcl = _{ ['\x21'..'\x39'] | ['\x3B'..'\x7F'] | ['\u{0080}'..'\u{07FF}'] | ['\u{0800}'..'\u{FFFF}']  }
        param = { nospcrlfcl ~ ( [":"] | nospcrlfcl )* }
        trailing_param = { ([":"] | [" "] | nospcrlfcl)* }
    }
}


pub fn parse_irc_line(line: &str) -> Option<IrcLine> {
    let mut parser = Rdp::new(StringInput::new(line));

    if !parser.expression() || !parser.end() {
        return None;
    }

    let builder = parser.queue().iter().fold(IrcLineBuilder::default(), |mut builder, token| {
        match token.rule {
            Rule::prefix => builder.prefix = Some(line[token.start..token.end].into()),
            Rule::command => builder.command = Some(line[token.start..token.end].into()),
            Rule::param |
            Rule::trailing_param => builder.args.push(line[token.start..token.end].into()),
            Rule::any | Rule::soi | Rule::eoi => {}
        };

        builder
    });

    let IrcLineBuilder { prefix, command, args } = builder;

    Some(IrcLine {
        prefix: prefix,
        command: command.expect("expected command").into(),
        args: args,
    })
}


#[derive(Debug, Clone, Copy)]
pub enum Numeric {
    RplWelcome = 1,
    RplChannelmodeis = 324,
    RplTopic = 332,
    RplEndofwho = 315,
    RplWhoreply = 352,
    RplNamreply = 353,
    RplEndofnames = 366,
    RplMotd = 372,
    RplMotdstart = 375,
    RplEndofmotd = 376,
    ErrNeedmoreparams = 461,
    ErrPasswdmismatch = 464,
}

impl Numeric {
    pub fn as_str(self) -> &'static str {
        self.into()
    }
}

impl<'a> From<Numeric> for &'a str {
    fn from(s: Numeric) -> &'a str {
        match s {
            Numeric::RplWelcome => "001",
            Numeric::RplChannelmodeis => "324",
            Numeric::RplTopic => "332",
            Numeric::RplEndofwho => "315",
            Numeric::RplWhoreply => "352",
            Numeric::RplNamreply => "353",
            Numeric::RplEndofnames => "366",
            Numeric::RplMotd => "372",
            Numeric::RplMotdstart => "375",
            Numeric::RplEndofmotd => "376",
            Numeric::ErrNeedmoreparams => "461",
            Numeric::ErrPasswdmismatch => "464",
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn simple_nick() {
        assert_eq!(parse_irc_line("NICK test"),
                   Some(IrcLine {
                       prefix: None,
                       command: Command::Nick,
                       args: vec!["test".into()],
                   }));

        assert_eq!("NICK test".parse().ok(),
                   Some(IrcCommand::Nick { nick: "test".into() }));
    }

    #[test]
    fn simple_user() {
        assert_eq!(parse_irc_line("USER test * * :Real Name"),
                   Some(IrcLine {
                       prefix: None,
                       command: Command::User,
                       args: vec!["test".into(), "*".into(), "*".into(), "Real Name".into()],
                   }));

        assert_eq!("USER test * * :Real Name".parse().ok(),
                   Some(IrcCommand::User {
                       user: "test".into(),
                       real_name: "Real Name".into(),
                   }));
    }

    #[test]
    fn simple_prefix() {
        assert_eq!(parse_irc_line(":example.com PrivMsg #test :Some text"),
                   Some(IrcLine {
                       prefix: Some("example.com".into()),
                       command: Command::PrivMsg,
                       args: vec!["#test".into(), "Some text".into()],
                   }));

        assert_eq!(":example.com PrivMsg #test :Some text".parse().ok(),
                   Some(IrcCommand::PrivMsg {
                       channel: "#test".into(),
                       text: "Some text".into(),
                   }));
    }

    #[test]
    fn simple_numeric() {
        assert_eq!(parse_irc_line("001 test :Some text"),
                   Some(IrcLine {
                       prefix: None,
                       command: Command::Numeric {
                           code: 1,
                           string: *b"001",
                       },
                       args: vec!["test".into(), "Some text".into()],
                   }))
    }
}