/* ********************************************************************************************************** */
/*                                                                                                            */
/*                                                     :::::::::  ::::::::   ::::::::   :::    ::: :::::::::: */
/* main.rs                                            :+:    :+: :+:    :+: :+:    :+: :+:    :+: :+:         */
/*                                                   +:+    +:+ +:+    +:+ +:+        +:+    +:+ +:+          */
/* By: codespace <marvin@doche.io>                  +#+    +:+ +#+    +:+ +#+        +#++:++#++ +#++:++#      */
/*                                                 +#+    +#+ +#+    +#+ +#+        +#+    +#+ +#+            */
/* Created: 2023/08/21 04:55:57 by codespace      #+#    #+# #+#    #+# #+#    #+# #+#    #+# #+#             */
/* Updated: 2023/08/21 04:56:01 by codespace     #########  ########   ########   ###    ### ##########.io    */
/*                                                                                                            */
/* ********************************************************************************************************** */

mod modules;
use std::env;

use modules::client::client;
use modules::peer::peer;
use modules::server::server;

#[tokio::main(flavor = "multi_thread")]
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
