pub mod config;
pub mod contract;
pub mod error;
pub mod messages;
pub mod phases;
pub mod setup;
pub mod state;
pub mod utils;

use anyhow::anyhow;
use console::style;
use error::AppError;
use messages::MessageType;
use setup::Setup;
use std::path::PathBuf;

use config::Config;
use phases::{
    actions_menu::ActionsMenuPhase, check_docker::DockerAvailablePhase,
    check_status::CheckStatusPhase, select_network::SelectNetworkPhase,
    select_node_ip::SelectNodeIP, select_private_key::SelectPrivateKeyPhase, Phase,
};
use utils::{
    config::{ConfigPath, JsonConfig},
    logger,
};

#[tokio::main]
async fn main() -> Result<(), AppError> {
    utils::set_heavy_panic();
    logger::init();

    let config = Config::load_json(PathBuf::from(&ConfigPath::Relative {
        root: "./",
        path: "./config/custom.json",
    }))?;

    let run_result = if utils::is_update_run() {
        run_update().await
    } else {
        run(&config).await
    };

    run_result.inspect_err(|e| {
        let _ = cliclack::log::error(e);
    })
}

async fn run(config: &Config) -> Result<(), AppError> {
    cliclack::clear_screen()?;

    print_intro()?;

    DockerAvailablePhase {}.run().await?;
    let mut state = state::State::read()?;

    let mut select_network = SelectNetworkPhase::new(state.network.as_ref(), &config.networks);
    select_network.run().await?;
    state.network = select_network.network.cloned();

    let mut select_private_key = SelectPrivateKeyPhase::new(state.private_key);
    select_private_key.run().await?;
    state.address = select_private_key
        .private_key
        .as_ref()
        .map(utils::secp256k1_signing_key_to_eth_address);
    state.private_key = select_private_key.private_key;

    let mut select_node_ip = SelectNodeIP::new(state.ip);
    select_node_ip.run().await?;
    state.ip = select_node_ip.node_ip;

    state.write()?;

    let setup = Setup::new(state)?;
    setup.run().await?;

    cliclack::log::step(MessageType::DockerStarting)?;

    utils::exec::run_docker_compose_up()?;

    cliclack::log::step(MessageType::DockerStarted)?;

    let web3_client_remote = web3::Web3::new(web3::transports::Http::new(&setup.network.rpc)?);
    let web3_client_local = web3::Web3::new(web3::transports::Http::new("http://127.0.0.1:8545")?);

    let mut check_status =
        CheckStatusPhase::new(web3_client_remote.clone(), &setup.network, setup.address).await?;
    check_status.run().await?;

    let mut actions_menu = ActionsMenuPhase::new(
        config.discord_webhook_url.clone(),
        web3_client_remote,
        web3_client_local,
    );
    loop {
        if actions_menu.quit {
            break;
        }

        actions_menu.run().await?;
    }

    Ok(())
}

async fn run_update() -> Result<(), AppError> {
    cliclack::clear_screen()?;

    let state = state::State::read()?;
    if !state.is_complete() {
        return Err(anyhow!("State is missing some data").into());
    }

    let setup = Setup::new(state)?;
    setup.run().await?;

    cliclack::log::step(MessageType::DockerStarting)?;

    utils::exec::run_docker_compose_pull()?;
    utils::exec::run_docker_compose_up()?;

    cliclack::log::step(MessageType::DockerStarted)?;

    Ok(())
}

fn print_intro() -> anyhow::Result<()> {
    cliclack::intro(
        style(
            r#"
                   ,lc,..                                                      
                    'cx0K0xoc,..                                                
                       .;o0WMWX0xoc,..                                          
                          .cOWMMMMMWX0xolc,..                                   
                            .lXMMMMMMMMMMMWX0koc,..                             
                              :XMMMMMMMMMMMMMMMMWX0koc,..                       
                              .dWMMMMMMMMMMMMMMMMMMMMMWX0ko;                    
                               oWMMMMMMMMMMMMMMMMMMMMMMMWN0o.                   
                              'OMMMMMMMMMMMMMMMMMMWN0kdc;..                     
                             'kWMMMMMMMMMMMMWX0kdc;..                           
                           .lKMMMMMMMWX0Okoc;..                                 
                        .,dKWMMWX0koc;..                                        
                     .:d0XX0koc;..                                              
                    ;xxoc;..                                                    
                                                                              
                                                                                
                                                                                
    10001      1x1   1000000001    1x000000001.      10001       .100000001.
   00  x00     1x1   10       001  1x1        1x1   00  x00     10        101 
  000xxxx000   1x1   1O0xxxx001    1x1        1x1  000xxxx000   101        01
 000      000  1x1   10     1111   1x100000001x   000      000   '100000001'
    "#,
        )
        .on_black()
        .blue(),
    )
    .map_err(anyhow::Error::from)
}
