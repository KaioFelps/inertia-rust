# Inertia + Actix with server-side rendering

This example uses `inertia_rust` crate with the following stack:
- **vite-rust** as assets manager;
- **actix-web** as http server;
- **Laravel vite-plugin** for Vite setup;
- **actix-files** for serving static assets;
- **Node.js** for server-side rendering.

## Running
#### Development
For running the project in **development** mode, you'll need to processes: one to
run Vite's dev server (`npm run dev`) and another for the Rust application (`cargo run`).

On development, all assets are served by Vite's development server, and `vite-rust`
generates tags referencing the vite-served assets.

#### Production
On production, you need first to bundle your front-end with the command`npm run build`.
Then, when starting your Rust application, `vite-rust` will fallback to "Manifest" mode
(since Vite's dev server is not running) and use the bundle's manifest file to generate
the tags locally.
