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

use modules::client::client;
use modules::peer::peer;
use modules::server::server;

#[tokio::main]
async fn main() {
    let mode = env::args().nth(1).expect("Unable to select mode");
    if mode.eq("peer") {
        peer().await;
    } else if mode.eq("server") {
        server().await;
    } else if mode.eq("client") {
        client().await;
    } else {
        eprintln!("Unable to select mode");
    }
}
