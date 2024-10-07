pub mod config;
pub mod error;
pub mod messages;
pub mod phases;
pub mod state;
pub mod utils;

use console::style;
use phases::{
    check_docker::DockerAvailablePhase, select_network::SelectNetworkPhase,
    select_node_ip::SelectNodeIP, select_private_key::SelectPrivateKeyPhase,
};
use utils::{config::JsonConfig, logger};

use crate::phases::Phase;
use config::Config;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    utils::set_heavy_panic();
    logger::init();

    let config = Config::load_json("./", "./config/custom.json")?;

    cliclack::clear_screen()?;

    print_intro()?;

    DockerAvailablePhase {}.run().await?;

    let mut state = state::State::read()?;

    let mut select_network = SelectNetworkPhase::new(state.network.as_ref(), &config.networks);
    select_network.run().await?;
    state.network = select_network.network.cloned();

    let mut select_private_key = SelectPrivateKeyPhase::new(state.private_key);
    select_private_key.run().await?;
    state.address = select_private_key.address();
    state.private_key = select_private_key.private_key;

    let mut select_node_ip = SelectNodeIP::new(state.ip);
    select_node_ip.run().await?;
    state.ip = select_node_ip.node_ip;

    state.write()?;

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
