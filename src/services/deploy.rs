use std::fs;
use std::fs::File;
use std::io::{BufRead, BufReader, Read, Write};
use std::net::TcpStream;
use std::path::Path;

use flate2::write::GzEncoder;
use flate2::Compression;
use globset::{Glob, GlobSetBuilder};
use ssh2::Session;
use uuid::Uuid;
use walkdir::WalkDir;

use super::types::ServiceResult;
use crate::api_client::{ApiClient, LaunchWorkerRequest, ServiceDeployRequest};
use crate::schemas::Service;
use crate::services::types::ServiceError;
use crate::utils::{await_exec_result, exec_command_with_output};

const BUILD_WORKER_USER: &str = "ubuntu";
const BUILD_LOCATION: &str = "_build";

/// This command must be run from the same location as the dockerfile of the service to be deployed.
/// First, builds an image and pushes it to ECR.
/// Second, triggers deploy of the service on the server.
pub fn deploy(api_client: &ApiClient, service: Service, build_args: Option<Vec<String>>) {
    debug!("got service: {:?}", service);
    let ecr_repo_uri = service.ecr_repo_url.unwrap();

    debug!("Building image and pushing to ECR: {}", ecr_repo_uri);

    let (success, version) =
        exec_command_with_output("git", vec!["rev-parse", "--short", "HEAD"]).unwrap();

    let version_sha = match success {
        true => sanitize_word(version),
        false => {
            eprintln!("Error getting git sha");
            return;
        }
    };
    println!("Version: {}", version_sha);

    debug!("version to be deployed: {}", version_sha);
    let (worker_slug, run_slug) = match api_client.launch_worker(
        &LaunchWorkerRequest {
            version: version_sha.trim().to_string(),
        },
        &service.slug,
    ) {
        Ok(resp) => (resp.build, resp.log),
        Err(err) => {
            eprintln!("Error: {}", err);
            return;
        }
    };

    if let Some(exec_log_slug) = run_slug {
        println!("Launching build worker: {}", service.name);
        if !await_exec_result(api_client, &exec_log_slug, None) {
            eprintln!("There was an error launching worker.");
            return;
        }
    }

    let worker = match api_client.get_worker_details(&worker_slug) {
        Ok(w) => w,
        Err(err) => {
            eprintln!("Error: {}", err);
            return;
        }
    };

    // create pem key file with read only permission
    let ssh_key_path = format!("{}.pem", &worker.ssh_key_name);
    if !Path::new(&ssh_key_path).exists() {
        let mut ssh_key_file = File::create(ssh_key_path.clone()).unwrap();
        ssh_key_file.write_all(worker.ssh_key.as_bytes()).unwrap();
        let mut permissions = ssh_key_file.metadata().unwrap().permissions();
        permissions.set_readonly(true);
        ssh_key_file.set_permissions(permissions).unwrap();
    }

    // prepare build directory
    match setup_deployment_dir() {
        Ok(()) => debug!("deployment dir is ready"),
        Err(err) => {
            eprintln!("Error: {}", err);
            return;
        }
    }
    // create tar.gz build directory
    let build_tarball = match create_build_tarball() {
        Ok(tarball) => {
            println!("Build tarballed ok");
            tarball
        }
        Err(err) => {
            eprintln!("Error: {}", err);
            return;
        }
    };

    let ssh_conn = match get_session(&worker.public_ip, "ubuntu", &ssh_key_path) {
        Ok(ss) => {
            debug!("connected to build worker");
            ss
        }
        Err(err) => {
            eprintln!("Error: {}", err);
            return;
        }
    };
    // upload tar.gz to worker server
    match upload_build_tarball_to_worker(&ssh_conn, &build_tarball) {
        Ok(()) => println!("Build uploaded to worker"),
        Err(err) => {
            eprintln!("Error: {}", err);
            return;
        }
    };

    // unpack the tarball on server
    match unpack_and_build_tarball(&ssh_conn, &build_tarball, build_args, &version_sha) {
        Ok(()) => println!("Build extracted"),
        Err(err) => {
            eprintln!("Error: {}", err);
            return;
        }
    }

    // trigger deploy service
    let run_slug = match api_client.deploy_service(
        &service.slug,
        &ServiceDeployRequest {
            version: version_sha.trim().to_string(),
        },
    ) {
        Ok(resp) => resp.log,
        Err(err) => {
            eprintln!("Error: {}", err);
            return;
        }
    };

    println!("Deploying service: {}", service.name);
    await_exec_result(api_client, &run_slug, None);
}

fn unpack_and_build_tarball(
    ssh_conn: &Session,
    build_tarball: &str,
    build_args: Option<Vec<String>>,
    version: &str,
) -> ServiceResult<()> {
    println!("Extracting build package");
    exec_cmd_on_server(
        ssh_conn,
        &format!("mkdir -p /home/{}/deployment", BUILD_WORKER_USER),
    )?;

    exec_cmd_on_server(
        ssh_conn,
        &format!(
            "tar -xzvf /tmp/{} -C /home/{}/deployment",
            build_tarball, BUILD_WORKER_USER
        ),
    )?;

    debug!("Removing build tarball");
    fs::remove_file(build_tarball)?;
    debug!("Removing build directory");
    fs::remove_dir_all(BUILD_LOCATION)?;

    let mut build_cmd = format!(
        "/home/{}/chiliseed-build-worker -v {}",
        BUILD_WORKER_USER, version
    );
    if let Some(args) = build_args {
        for arg in args {
            build_cmd.push_str(&format!(" --build-arg {}", arg));
        }
    }
    let exec_status = exec_cmd_on_server(ssh_conn, &build_cmd)?;
    if exec_status != 0 {
        return Err(ServiceError::DeploymentError(
            "Build script failed".to_string(),
        ));
    }

    Ok(())
}

fn create_build_tarball() -> ServiceResult<String> {
    let uuid = Uuid::new_v4();
    let build_tar_name = format!("build_{}.tar.gz", uuid.to_simple());
    let build_tar = File::create(build_tar_name.clone())?;
    let encoder = GzEncoder::new(build_tar, Compression::default());
    let mut tar = tar::Builder::new(encoder);
    tar.append_dir_all("build", BUILD_LOCATION)?;
    Ok(build_tar_name)
}

fn upload_build_tarball_to_worker(ssh_conn: &Session, build_tarball: &str) -> ServiceResult<()> {
    println!("Uploading {} to build worker", build_tarball);
    let mut deployment_package_fp = File::open(build_tarball.clone())?;
    let pck_meta = deployment_package_fp.metadata()?;
    let mut channel = ssh_conn.scp_send(
        Path::new(&format!("/tmp/{}", build_tarball)),
        0o644,
        pck_meta.len(),
        None,
    )?;

    loop {
        let mut buffer = Vec::new();
        let read_bytes = std::io::Read::by_ref(&mut deployment_package_fp)
            .take(1000)
            .read_to_end(&mut buffer)?;
        if read_bytes == 0 {
            break;
        }
        channel.write(&buffer)?;
    }

    Ok(())
}

fn get_session(server_ip: &str, server_user: &str, ssh_key: &str) -> ServiceResult<Session> {
    let tcp = TcpStream::connect(format!("{}:22", server_ip))?;
    let mut sess = Session::new()?;
    sess.set_tcp_stream(tcp);
    sess.handshake()?;
    sess.userauth_pubkey_file(server_user, None, &Path::new(&ssh_key), None)?;
    Ok(sess)
}

fn exec_cmd_on_server(ssh_conn: &Session, cmd: &str) -> ServiceResult<i32> {
    println!("[remote]: {}", cmd);
    let mut channel = ssh_conn.channel_session()?;
    channel.exec(cmd).unwrap();
    loop {
        let mut buffer = Vec::new();
        let n = std::io::Read::by_ref(&mut channel)
            .take(10)
            .read_to_end(&mut buffer)
            .unwrap();
        if n == 0 {
            let mut s = String::new();
            channel.stderr().read_to_string(&mut s).unwrap();
            eprintln!("{}", s);
            break;
        }
        print!("{}", String::from_utf8_lossy(&buffer));
    }
    channel.wait_close().unwrap();
    Ok(channel.exit_status().unwrap())
}

fn setup_deployment_dir() -> ServiceResult<()> {
    if Path::new(BUILD_LOCATION).exists() {
        fs::remove_dir_all(BUILD_LOCATION)?;
    }

    fs::create_dir(BUILD_LOCATION)?;

    let gitignore = File::open(".gitignore")?;
    let mut ignores: Vec<String> = BufReader::new(gitignore)
        .lines()
        .filter_map(|line| line.ok())
        .filter(|line| !line.trim().is_empty())
        .collect();

    ignores.push("*.pem".to_string());
    ignores.push(".git/*".to_string());
    ignores.push("_build/*".to_string());
    ignores.push("*.tar.gz".to_string());

    let mut path_checker = GlobSetBuilder::new();
    ignores.iter().for_each(|ignore_pattern| {
        let mut clean_ignore = ignore_pattern.trim().to_string();
        if clean_ignore.starts_with("/") {
            debug!("Adding .{} to ignore", clean_ignore);
            clean_ignore = ".".to_string() + &clean_ignore;
        } else if !clean_ignore.starts_with("./") {
            debug!("Adding ./{} to ignore", clean_ignore);
            clean_ignore = "./".to_string() + &clean_ignore;
        }
        if Path::new(&clean_ignore).is_dir() {
            debug!("Adding * to {} ignore", clean_ignore);
            clean_ignore = clean_ignore + "/*";
        }
        debug!("Ignoring path: {}", clean_ignore);
        path_checker.add(Glob::new(&clean_ignore).unwrap());
    });

    let set_path_checker = path_checker.build()?;

    for entry in WalkDir::new(".")
        .follow_links(true)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let path = entry.path();
        if path.is_dir() {
            continue;
        }

        let matched_patterns_idx = set_path_checker.matches(path);
        if !matched_patterns_idx.is_empty() {
            continue;
        }

        let move_to = format!("{}/{}", &BUILD_LOCATION, path.to_str().unwrap());
        let build_path = Path::new(&move_to);

        fs::create_dir_all(build_path.parent().unwrap())?;
        fs::copy(path, build_path)?;
    }
    Ok(())
}

/// Remove whitespaces and trailing new line or carriage signs
fn sanitize_word(word: Vec<u8>) -> String {
    let word_utf = String::from_utf8(word).unwrap();
    let mut word_utf = String::from(word_utf.trim());
    if word_utf.ends_with("\n") {
        word_utf.pop();
        if word_utf.ends_with("\r") {
            word_utf.pop();
        }
    }
    word_utf
}
