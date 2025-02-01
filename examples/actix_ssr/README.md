# Inertia + Actix with server-side rendering

This example uses `inertia_rust` crate with the following stack:
- **vite-rust** as assets manager;
- **actix-web** as http server;
- **Laravel vite-plugin** for Vite setup;
- **actix-files** for serving static assets;
- **Node.js** for server-side rendering.

## What to learn with this example app?
Within this example project, you can learn how to:

- Setup Inertia Rust + Vite Rust + Template Resolver
    - src/config/inertia.rs
    - src/config/vite.rs
    - www/root.hbs

- Setup the most important middlewares to extract the full power from Inertia Rust + Actix Web:
    - src/config/file_session.rs
    - src/middlewares/garbage_collector.rs
    - src/middlewares/reflash_temporary_session.rs
    - src/server.rs

- Get an idea of how to serve your bundle assets on production
    - src/server.rs
    - src/config/vite.rs

- Setup Vite for development and production
    - vite.config.js

## Running
#### Development
For running the project in **development** mode, you'll need to processes: one to
run Vite's dev server (`npm run dev`) and another for the Rust application (`npx vite build -ssr && cargo run`).

On development, all assets are served by Vite's development server, and `vite-rust`
generates tags referencing the vite-served assets.

#### Production
On production, you need first to bundle your front-end with the command`npm run build`.
Then, when starting your Rust application, `vite-rust` will fallback to "Manifest" mode
(since Vite's dev server is not running) and use the bundle's manifest file to generate
the tags locally.
