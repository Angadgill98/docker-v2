use std::env;
use std::io::{self, Write};

use crate::client::Client;

pub async fn run(client: &mut Client) {
    loop {
        print!("docker-cli> ");
        io::stdout().flush().unwrap();

        let mut input = String::new();

        match io::stdin().read_line(&mut input) {
            Ok(0) => {
                // EOF / Ctrl+D
                break;
            }

            Ok(_) => {}

            Err(e) => {
                eprintln!("Input Error: {}", e);
                break;
            }
        }

        let input = input.trim();

        if input.is_empty() {
            continue;
        }

        // Exit CLI
        if input == "exit" || input == "quit" {
            break;
        }

        // Help
        if input == "help" {
            print_help();
            continue;
        }

        // Convert command line into arguments
        let args: Vec<String> = input
            .split_whitespace()
            .map(|s| s.to_string())
            .collect();

        let request = match build_request(&args) {
            Ok(request) => request,

            Err(e) => {
                eprintln!("CLI Error: {}", e);
                continue;
            }
        };

        if let Err(e) = client.send(request).await {
            eprintln!("Send Error: {:?}", e);
            break;
        }
    }
}

fn add_field(
    buf: &mut Vec<u8>,
    data: &[u8],
) {
    let len = (data.len() as u64).to_be_bytes();

    buf.extend_from_slice(&len);
    buf.extend_from_slice(data);
}

fn build_request(
    args: &[String],
) -> Result<Vec<u8>, String> {

    if args.is_empty() {
        return Err("No command provided".into());
    }

    let command = args[0].as_str();

    let mut payload = Vec::new();

    match command {

        // -----------------------------------------
        // create_bridge
        // -----------------------------------------

        "create_bridge" => {

            if args.len() != 4 {
                return Err(
                    "Usage: create_bridge <name> <ip> <prefix>"
                        .into()
                );
            }

            add_field(
                &mut payload,
                b"create_bridge",
            );

            add_field(
                &mut payload,
                args[1].as_bytes(),
            );

            add_field(
                &mut payload,
                args[2].as_bytes(),
            );

            let prefix: u8 =
                args[3]
                    .parse()
                    .map_err(|_| {
                        "Invalid prefix".to_string()
                    })?;

            payload.push(prefix);
        }

        // -----------------------------------------
        // create_veth
        // -----------------------------------------

        "create_veth" => {

            if args.len() != 3 {
                return Err(
                    "Usage: create_veth <front> <back>"
                        .into()
                );
            }

            add_field(
                &mut payload,
                b"create_veth",
            );

            add_field(
                &mut payload,
                args[1].as_bytes(),
            );

            add_field(
                &mut payload,
                args[2].as_bytes(),
            );
        }

        // -----------------------------------------
        // create_container
        // -----------------------------------------

        "create_container" => {

            if args.len() != 4 {
                return Err(
                    "Usage: create_container <path> <arguments> <name>"
                        .into()
                );
            }

            add_field(
                &mut payload,
                b"create container",
            );

            add_field(
                &mut payload,
                args[1].as_bytes(),
            );

            add_field(
                &mut payload,
                args[2].as_bytes(),
            );

            add_field(
                &mut payload,
                args[3].as_bytes(),
            );
        }

        // -----------------------------------------
        // Unknown command
        // -----------------------------------------

        _ => {
            return Err(
                format!(
                    "Unknown command: {}",
                    command
                )
            );
        }
    }

    // -----------------------------------------
    // Final packet
    //
    // [8 byte total payload length]
    // [payload]
    // -----------------------------------------

    let mut request = Vec::new();

    let len =
        (payload.len() as u64)
            .to_be_bytes();

    request.extend_from_slice(&len);
    request.extend_from_slice(&payload);

    Ok(request)
}

fn print_help() {
    println!(
        r#"
Available commands:

  create_bridge <name> <ip> <prefix>
      Create a Linux bridge.

  create_veth <front> <back>
      Create a veth pair.

  create_container <path> <arguments> <name>
      Create a container.

  help
      Show this help.

  exit
      Exit the CLI.
"#
    );
}