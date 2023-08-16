/* ********************************************************************************************************** */
/*                                                                                                            */
/*                                                     :::::::::  ::::::::   ::::::::   :::    ::: :::::::::: */
/* main.rs                                            :+:    :+: :+:    :+: :+:    :+: :+:    :+: :+:         */
/*                                                   +:+    +:+ +:+    +:+ +:+        +:+    +:+ +:+          */
/* By: se-yukun <yukun@doche.io>                    +#+    +:+ +#+    +:+ +#+        +#++:++#++ +#++:++#      */
/*                                                 +#+    +#+ +#+    +#+ +#+        +#+    +#+ +#+            */
/* Created: 2023/08/16 21:18:22 by se-yukun       #+#    #+# #+#    #+# #+#    #+# #+#    #+# #+#             */
/* Updated: 2023/08/16 21:18:27 by se-yukun      #########  ########   ########  ###    ### ##########.io.    */
/*                                                                                                            */
/* ********************************************************************************************************** */

mod modules;
use std::env;

use modules::peer::peer;
use modules::server::server;
use modules::client::client;

#[tokio::main]
async fn main() {
    let mode = env::args().nth(2).expect("Unable to select mode");
    if mode == "peer" {
        peer().await;
    } else if mode == "server" {
        server().await;
    } else if mode == "client" {
        client().await;
    } else {
        eprintln!("Unable to select mode");
    }
}
