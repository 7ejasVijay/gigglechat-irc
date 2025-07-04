//CAP LS 302
//JOIN :
//NICK leo
//USER leonardosantangelo leonardosantangelo localhost :Leonardo Santangelo

#[derive(Debug)]
pub struct NickArgs {
    nickname: String
}

#[derive(Debug)]
pub enum Command {
    Nick(NickArgs)
}

pub fn parse_command(irc_string: &str) -> Result<Command, anyhow::Error> {
    let irc_string = irc_string.trim_end();
    let irc_string_len = irc_string.len();
    let first_space_idx = irc_string.find(' ');
    println!("{irc_string}");
    println!("{irc_string_len}");
    println!("{first_space_idx:?}");

    let (command_name, arg_string) = match first_space_idx {
        None => (irc_string, ""),
        Some(idx) => {
            let arg_string = irc_string[idx + 1..].trim_end();
            let command_name =  irc_string[..idx].trim_end();
            (command_name, arg_string)
        }
    };

    match command_name {
        "NICK" => Ok(Command::Nick(parse_nick_args(arg_string)?)),
        _ => Err(anyhow::anyhow!("Unknown command: {}", command_name))
    }
}

pub fn parse_nick_args(arg_string: &str) -> Result<NickArgs, anyhow::Error> {
    // Split the argument string to an argument vector according to IRC syntax
    let args: Vec<&str> = match arg_string.split_once(" :") {
        None => arg_string.split(' ').collect(),
        Some((first_args, last_arg)) => {
            let mut args: Vec<&str> = first_args.split(' ').collect();
            args.push(last_arg);
            args
        }
    };
    println!("{:?}", args);

    let mut args = args.into_iter();
    println!("{:?}", args);

    // Get the next arg (in this case, the nickname)
    let arg_string = args.next().ok_or(anyhow::anyhow!("Not enough arguments"))?;

    // Parse it to an owned string
    let nickname = arg_string.parse::<String>()?;

    Ok(NickArgs { nickname })
}

fn main() {
    let result = parse_command("NICK there all");
    println!("{result:?}");
}
