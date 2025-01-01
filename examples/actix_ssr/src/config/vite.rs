use vite_rust::{Vite, ViteConfig};

use crate::ASSETS_VERSION;

pub async fn initialize_vite() -> Vite {
    let vite_config = ViteConfig::default().set_manifest_path("public/bundle/manifest.json");

    match Vite::new(vite_config).await {
        Err(err) => panic!("{}", err),
        Ok(vite) => {
            let _ = ASSETS_VERSION.set(vite.get_hash().unwrap().to_string().leak());
            vite
        }
    }
}
