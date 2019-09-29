use iron::prelude::*;
use mount::Mount;
use playground_middleware::{
    Cache, FileLogger, GuessContentType, ModifyWith, Staticfile, StatisticLogger,
};
use serde::{de::DeserializeOwned, Serialize};
use serde_derive::{Deserialize, Serialize};
use snafu::{ResultExt, Snafu};
use std::{
    any::Any,
    env,
    fs::File,
    io::{BufWriter, Write},
    path::PathBuf,
    process::Command,
    time::Duration,
};
use tempdir::TempDir;

const DEFAULT_ADDRESS: &str = "127.0.0.1";
const DEFAULT_PORT: u16 = 5000;
const DEFAULT_LOG_FILE: &str = "access-log.csv";

fn main() {
    let _ = dotenv::dotenv();
    env_logger::init();

    // Determine the environment parameters.
    let root: PathBuf = env::var_os("LLHD_WEBSITE_ROOT")
        .expect("Must specify LLHD_WEBSITE_ROOT")
        .into();
    let address = env::var("LLHD_WEBSITE_ADDRESS").unwrap_or_else(|_| DEFAULT_ADDRESS.to_string());
    let port = env::var("LLHD_WEBSITE_PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(DEFAULT_PORT);
    let log_file =
        env::var("LLHD_WEBSITE_LOG_FILE").unwrap_or_else(|_| DEFAULT_LOG_FILE.to_string());

    // Assemble the static files to be served.
    let files = Staticfile::new(&root).expect("Unable to open root directory");
    let mut files = Chain::new(files);
    let one_day = Duration::new(60 * 60 * 24, 0);
    // files.link_after(ModifyWith::new(Cache::new(one_day)));
    files.link_after(GuessContentType::new(iron::headers::ContentType::html().0));

    // Assemble the routes.
    let mut mount = Mount::new();
    mount.mount("/", files);
    mount.mount("/compile", compile);

    // Assemble the server chain.
    let mut chain = Chain::new(mount);
    chain.link_around(StatisticLogger::new(
        FileLogger::new(log_file).expect("Unable to create file logger"),
    ));

    // Start the web server.
    log::info!("Starting the server on http://{}:{}", address, port);
    Iron::new(chain)
        .http((&*address, port))
        .expect("Unable to start server");
}

/// All possible errors that may occur.
#[derive(Debug, Snafu)]
enum Error {
    // Request handling
    #[snafu(display("Unable to serialize response: {}", source))]
    Serialization { source: serde_json::Error },
    #[snafu(display("Unable to deserialize request: {}", source))]
    Deserialization { source: bodyparser::BodyError },
    #[snafu(display("No request was provided"))]
    RequestMissing,
    // Sandbox
    #[snafu(display("Unable to create temporary directory: {}", source))]
    UnableToCreateTempDir { source: std::io::Error },
    #[snafu(display("Unable to create source file: {}", source))]
    UnableToCreateSourceFile { source: std::io::Error },
    #[snafu(display("Unable to find a module in the input"))]
    UnableToFindModule,
    #[snafu(display("Unable to execute the compiler: {}", source))]
    UnableToExecuteCompiler { source: std::io::Error },
    #[snafu(display("Unable to read output file: {}", source))]
    UnableToReadOutput { source: std::io::Error },
    #[snafu(display("Output was not valid UTF-8: {}", source))]
    OutputNotUtf8 { source: std::string::FromUtf8Error },
}

/// The result type carrying the above error.
type Result<T> = std::result::Result<T, Error>;

/// The fallback JSON error response in case everything goes wrong.
const FATAL_ERROR_JSON: &str =
    r#"{"error": "Multiple cascading errors occurred, abandon all hope"}"#;

/// The error response.
#[derive(Debug, Clone, Serialize)]
struct ErrorJson {
    error: String,
}

/// The compilation request issued by the frontend.
#[derive(Debug, Clone, Deserialize)]
struct CompileRequest {
    code: String,
}

/// The compilation response issued by the server.
#[derive(Debug, Clone, Serialize)]
struct CompileResponse {
    output: String,
}

fn compile(req: &mut Request<'_, '_>) -> IronResult<Response> {
    serialize_to_response(deserialize_from_request(req, compile_handler))
}

fn compile_handler(req: CompileRequest) -> Result<CompileResponse> {
    Sandbox::new()?.compile(&req).map(CompileResponse::from)
}

/// Deserialize a request from a request body.
fn deserialize_from_request<Req, Resp, F>(req: &mut Request<'_, '_>, f: F) -> Result<Resp>
where
    F: FnOnce(Req) -> Result<Resp>,
    Req: DeserializeOwned + Clone + Any + 'static,
{
    let body = req
        .get::<bodyparser::Struct<Req>>()
        .context(Deserialization)?;
    let req = body.ok_or(Error::RequestMissing)?;
    let resp = f(req)?;
    Ok(resp)
}

/// Serialize an object or error to JSON and return an HTTP response.
fn serialize_to_response<Resp>(response: Result<Resp>) -> IronResult<Response>
where
    Resp: Serialize,
{
    // Serialize the response to JSON in case it is `Ok(..)`.
    let response = response.and_then(|resp| {
        let resp = serde_json::ser::to_string(&resp).context(Serialization)?;
        Ok(resp)
    });

    // Convert the response object to an actual HTTP response.
    match response {
        Ok(body) => Ok(Response::with((
            iron::status::Ok,
            iron::modifiers::Header(iron::headers::ContentType::json()),
            body,
        ))),
        Err(err) => {
            let err = ErrorJson {
                error: err.to_string(),
            };
            match serde_json::ser::to_string(&err) {
                Ok(error_str) => Ok(Response::with((
                    iron::status::InternalServerError,
                    iron::modifiers::Header(iron::headers::ContentType::json()),
                    error_str,
                ))),
                Err(_) => Ok(Response::with((
                    iron::status::InternalServerError,
                    iron::modifiers::Header(iron::headers::ContentType::json()),
                    FATAL_ERROR_JSON,
                ))),
            }
        }
    }
}

fn vec_to_str(v: Vec<u8>) -> Result<String> {
    String::from_utf8(v).context(OutputNotUtf8)
}

lazy_static::lazy_static! {
    static ref MODULE_REGEX: regex::Regex = regex::Regex::new(r"\bmodule\s+([a-zA-Z_]+)\b").unwrap();
}

/// A sandbox where the compiler and simulator are executed.
struct Sandbox {
    #[allow(dead_code)]
    scratch: TempDir,
    input_file: PathBuf,
}

impl Sandbox {
    /// Create a new sandbox with the necessary directories.
    pub fn new() -> Result<Self> {
        let scratch = TempDir::new("llhd-io").context(UnableToCreateTempDir)?;
        let input_file = scratch.path().join("input.sv");

        Ok(Sandbox {
            scratch,
            input_file,
        })
    }

    /// Compile a piece of HDL code.
    pub fn compile(&self, req: &CompileRequest) -> Result<CompileResponse> {
        // Try to determine the name of the module to be compiled.
        let module_name = match MODULE_REGEX
            .captures(&req.code)
            .and_then(|c| c.get(1))
            .map(|m| m.as_str())
        {
            Some(f) => f,
            None => return Err(Error::UnableToFindModule),
        };

        // Write the source code and execute the compiler.
        self.write_source_code(&req.code)?;
        let mut command = self.compile_command(module_name);
        let output = command.output().context(UnableToExecuteCompiler)?;

        // Capture the compiler output.
        let stdout = vec_to_str(output.stdout)?;
        let stderr =
            vec_to_str(strip_ansi_escapes::strip(output.stderr).context(UnableToReadOutput)?)?;

        Ok(CompileResponse {
            output: format!("{}\n{}", stdout, stderr),
        })
    }

    /// Write input source code into the sandbox.
    fn write_source_code(&self, code: &str) -> Result<()> {
        let data = code.as_bytes();
        let file = File::create(&self.input_file).context(UnableToCreateSourceFile)?;
        let mut file = BufWriter::new(file);
        file.write_all(data).context(UnableToCreateSourceFile)?;
        log::debug!(
            "Wrote {} bytes of source to {}",
            data.len(),
            self.input_file.display()
        );
        Ok(())
    }

    /// Assemble the command to be executed to compile HDL to LLHD.
    fn compile_command(&self, module_name: &str) -> Command {
        let use_docker = true;
        let mut cmd = if use_docker {
            let mut cmd = self.docker_command();
            cmd.arg("llhd-sandbox"); // container name
            cmd.args(&["moore", "input.sv"]);
            cmd
        } else {
            let mut cmd = Command::new("moore");
            cmd.arg(&self.input_file);
            cmd
        };
        cmd.args(&["-e", module_name]);
        log::debug!("Compilation command is {:?}", cmd);
        cmd
    }

    /// Assemble a docker command that deals with the given input and output.
    fn docker_command(&self) -> Command {
        let mut mount_dir = self.input_file.as_os_str().to_os_string();
        mount_dir.push(":");
        mount_dir.push("/playground/");
        mount_dir.push(self.input_file.file_name().unwrap());

        let mut cmd = basic_secure_docker_command();
        cmd.arg("--volume").arg(&mount_dir);
        cmd
    }
}

/// Assemble the basic command to launch a secure docker container.
fn basic_secure_docker_command() -> Command {
    let mut cmd = Command::new("docker");
    cmd.arg("run");
    cmd.arg("--rm");
    cmd.arg("--cap-drop=ALL");
    cmd.arg("--cap-add=DAC_OVERRIDE");
    cmd.arg("--security-opt=no-new-privileges");
    cmd.args(&["--workdir", "/playground"]);
    cmd.args(&["--net", "none"]);
    cmd.args(&["--memory", "256m"]);
    cmd.args(&["--memory-swap", "320m"]);
    cmd.args(&["--env", "PLAYGROUND_TIMEOUT=10"]);
    cmd.args(&["--pids-limit", "512"]);
    cmd
}
