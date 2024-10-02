use std::error::Error;
use std::path::Path;
use std::process::{Child, Command};
use std::{fmt, io};

#[derive(Debug, Clone)]
pub struct NodeJsError {
    cause: String,
    description: String,
}

impl NodeJsError {
    pub fn new(cause: String, description: String) -> Self {
        NodeJsError {
            cause,
            description,
        }
    }
}

impl fmt::Display for NodeJsError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "cause: {};\ndescription: {}", self.cause, self.description)
    }
}

impl Error for NodeJsError {
    fn description(&self) -> &str {
        &self.description
    }
}

#[derive(Debug)]
pub struct NodeJsProc {
    child: Child,
    server: String
}

impl NodeJsProc {
    /// Starts a Node.js child process that runs an Inertia ssr file.
    ///
    /// # Arguments
    /// * `server_path`     - The path to the `ssr.js` file. E.g. "dist/server/ssr.js".
    /// * `server_url`      - The url where the server is running.
    ///
    /// # Errors
    /// Returns an [`NodeJsError`] if it fails to start the process, e.g. if the machine do not have
    /// [node] installed.
    ///
    /// [node]: https://nodejs.org
    ///
    /// # Return
    /// Returns an `NodeJsProc` instance. Note that you should call the `NodeJsProc::kill(self)`
    /// method before the application fully shuts down, or else the Node.js process will keep alive.
    ///
    /// # Example
    /// ```rust
    /// use inertia_rs::node_process::{NodeJsError, NodeJsProc};
    ///
    /// async fn server() {
    ///     let node = NodeJsProc::start("dist/server/ssr.js".into(), "localhost:15000".into());
    ///
    ///     if node.is_err() {
    ///         let err: NodeJsError = node.unwrap_err();
    ///         panic!("Failed to start node server: {}", err);
    ///     }
    ///
    ///     let node = node.unwrap();
    ///
    ///     // runs the server asynchronously,blocking the function on .await
    ///     // when the server stops running, don't forget to:
    ///     let _ = node.kill();
    /// }
    /// ```
    pub fn start(server_path: String, server_url: String) -> Result<Self, NodeJsError> {
        let path = Path::new(&server_path);

        if !path.exists() {
            return Err(NodeJsError::new("Invalid path".into(), format!("Server javascript file not found in {}.", &server_path)))
        }

        let string_path = match path.to_str() {
            None => return Err(NodeJsError::new("Invalid path".into(), "The given path contains invalid UTF-8 characters.".into())),
            Some(path) => path,
        };

        let child = match Command::new("node")
            .arg(string_path)
            .spawn() {
            Err(err) => return Err(NodeJsError::new(
                "Process error".into(),
                format!("Something went wrong on invoking a node server: {}", err.to_string())
            )),
            Ok(child) => child
        };

        Ok(NodeJsProc {
            child,
            server: server_url
        })
    }

    /// Kills the current Node.js process.
    pub async fn kill(self) -> io::Result<()> {
        let resp = reqwest::Client::new()
            .get(format!("{}/shutdown", self.server))
            .send()
            .await;

        if resp.is_err() {
            let _ = self.force_kill();
        }

        Ok(())
    }

    fn force_kill(mut self) -> io::Result<()> {
        self.child.kill()
    }
}
