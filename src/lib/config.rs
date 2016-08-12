use std::error::Error;
use std::collections::HashMap;
use std::io::Read;
use std::fs::File;
use yaml_rust::{Yaml, YamlLoader};
use yaml_rust::yaml::Hash;

#[derive(Clone, Debug)]
pub struct Host {
    pub name: String,
    pub user: String,
    pub build_dir: String,
    pub host: Option<String>,
    pub identity_file: Option<String>,
}

impl Host {
    fn host(&self) -> &str {
        self.host.as_ref().unwrap_or(&self.name)
    }

    pub fn ssh_command(&self, cmd: &str) -> Vec<String> {
        let mut args = Vec::new();

        args.push("-o".into());
        args.push("PreferredAuthentications=publickey".into());

        if let Some(ref identity_file) = self.identity_file {
            args.push("-i".into());
            args.push(identity_file.to_string());
        }

        args.push(format!("{}@{}", self.user, self.host()));
        args.push("-C".into());
        args.push(cmd.into());
        args
    }

    pub fn git_ssh_command(&self) -> Option<String> {
        if let Some(ref identity_file) = self.identity_file {
            Some(format!("ssh -i \"{}\"", identity_file))
        } else {
            None
        }
    }

    pub fn git_ssh_url(&self) -> String {
        format!("{}@{}:{}", self.user, self.host(), self.build_dir)
    }
}

pub type Hosts = HashMap<String, Host>;

#[derive(Debug)]
pub struct Config {
    pub hosts: Hosts,
    pub build: Vec<String>,
}

impl Config {
    pub fn new(hosts: Hosts, build: Vec<String>) -> Self {
        Config {
            hosts: hosts,
            build: build,
        }
    }
}

fn get_optional_str(hash: &Hash, host: &str, name: &str) -> Result<Option<String>, Box<Error>> {
    let key = Yaml::String(name.into());

    match hash.get(&key) {
        None => Ok(None),
        Some(&Yaml::String(ref value)) => Ok(Some(value.to_string())),
        Some(_) => Err(format!("invalid value for \"{}\" in host \"{}\"", name, host).into()),
    }
}

fn get_str(hash: &Hash, host: &str, name: &str) -> Result<String, Box<Error>> {
    match get_optional_str(hash, host, name) {
        Ok(Some(string)) => Ok(string),
        Err(err) => Err(err),
        _ => Err(format!("missing value for \"{}\" in host \"{}\"", name, host).into()),
    }
}

fn parse_host(name: &str, yaml: &Yaml) -> Result<Host, Box<Error>> {
    let hash = try!(yaml.as_hash().ok_or(format!("invalid configuration for host \"{}\"", name)));

    // TODO: should complain on unknown keys

    Ok(Host {
        name: name.into(),
        user: try!(get_str(&hash, &name, "user")),
        build_dir: try!(get_str(&hash, &name, "build_dir")),
        host: try!(get_optional_str(&hash, &name, "host")),
        identity_file: try!(get_optional_str(&hash, &name, "identity_file")),
    })
}

fn parse_hosts(yaml: &Yaml) -> Result<Hosts, Box<Error>> {
    if yaml.is_badvalue() {
        return Err("missing \"hosts\" configuration".into());
    }

    let hash = try!(yaml.as_hash().ok_or("invalid \"hosts\" configuration"));
    let mut hosts = Hosts::new();

    for (key, value) in hash {
        let name = try!(key.as_str().ok_or("\"hosts\" keys must be strings"));
        hosts.insert(name.into(), try!(parse_host(name, value)));
    }

    Ok(hosts)
}

fn parse_build(yaml: &Yaml) -> Result<Vec<String>, Box<Error>> {
    if yaml.is_badvalue() {
        return Err("missing \"build\" configuration".into());
    }

    if let Some(cmd) = yaml.as_str() {
        return Ok(vec![cmd.into()]);
    }

    let err_msg = "\"build\" configuration must be a string or an array of strings";

    if let Some(cmds) = yaml.as_vec() {
        let mut result = Vec::new();

        for cmd in cmds {
            result.push(try!(cmd.as_str().ok_or(err_msg)).into());
        }

        return Ok(result);
    }

    return Err(err_msg.into());
}

fn parse_config(contents: &str) -> Result<Config, Box<Error>> {
    let yaml = try!(YamlLoader::load_from_str(&contents));

    if yaml.len() == 0 {
        return Err("no configuration found".into());
    }

    let settings = &yaml[0];
    let hosts = try!(parse_hosts(&settings["hosts"]));
    let build = try!(parse_build(&settings["build"]));

    Ok(Config::new(hosts, build))
}

fn parse_config_file(name: &str) -> Result<Config, Box<Error>> {
    let mut file = try!(File::open(name));

    let mut contents = String::new();
    try!(file.read_to_string(&mut contents));

    parse_config(&contents)
}

pub fn read() -> Result<Config, Box<Error>> {
    match parse_config_file("bran.yml") {
        Ok(config) => Ok(config),
        Err(err) => Err(format!("Failed to read bran.yml: {}", err.description()).into()),
    }
}

#[cfg(test)]
mod tests {
    use super::parse_config;

    fn check_fail(contents: &str, msg: &str) {
        let config = parse_config(contents);
        assert!(config.is_err(), "parse_config should have failed");
        assert_eq!(config.unwrap_err().description(), msg);
    }

    #[test]
    fn fails_when_empty() {
        check_fail("", "no configuration found");
    }

    #[test]
    fn fails_when_hosts_missing() {
        let yaml = "hodor: 1";
        check_fail(yaml, "missing \"hosts\" configuration");
    }

    #[test]
    fn fails_when_hosts_invalid() {
        let yaml = "hosts: 1";
        check_fail(yaml, "invalid \"hosts\" configuration");
    }

    #[test]
    fn fails_when_host_name_is_not_string() {
        let yaml = "hosts: {[1, 2]: 3}";
        check_fail(yaml, "\"hosts\" keys must be strings");
    }

    #[test]
    fn fails_when_host_invalid() {
        let yaml = "hosts: {foo: 1}";
        check_fail(yaml, "invalid configuration for host \"foo\"");
    }

    #[test]
    fn fails_when_host_missing_build_dir() {
        let yaml = "hosts: {foo: {user: hodor}}";
        check_fail(yaml, "missing value for \"build_dir\" in host \"foo\"");
    }

    #[test]
    fn fails_when_host_invalid_build_dir() {
        let yaml = "hosts: {foo: {build_dir: 1, user: hodor}}";
        check_fail(yaml, "invalid value for \"build_dir\" in host \"foo\"");
    }

    #[test]
    fn fails_when_build_missing() {
        let yaml = "hosts: {}";
        check_fail(yaml, "missing \"build\" configuration");
    }

    #[test]
    fn fails_when_build_invalid() {
        let yaml = "{hosts: {}, build: {}}";
        check_fail(yaml,
                   "\"build\" configuration must be a string or an array of strings");
    }

    #[test]
    fn fails_when_build_not_string() {
        let yaml = "{hosts: {}, build: [1, 2, 3]}";
        check_fail(yaml,
                   "\"build\" configuration must be a string or an array of strings");
    }

    #[test]
    fn parses_build_string() {
        let yaml = "{hosts: {}, build: abc}";
        let config = parse_config(yaml).expect("should parse successfully");
        assert_eq!(config.build, ["abc"]);
    }

    #[test]
    fn parses_successfully() {
        let yaml = "
            hosts:
                win:
                    host: a
                    user: b
                    identity_file: c
                    build_dir: d
            build:
                - x
                - y
                - z";

        let config = parse_config(yaml).expect("should parse successfully");

        let win = &config.hosts["win"];
        assert_eq!(win.name, "win");
        assert_eq!(win.host, Some("a".into()));
        assert_eq!(win.user, "b");
        assert_eq!(win.identity_file, Some("c".into()));
        assert_eq!(win.build_dir, "d");

        assert_eq!(config.build, ["x", "y", "z"]);
    }

    #[test]
    fn git_ssh_command_with_identity_file() {
        let host = super::Host {
            name: "hodor".into(),
            user: "user".into(),
            build_dir: "build_dir".into(),
            host: None,
            identity_file: Some("id_rsa".into()),
        };

        assert_eq!(host.git_ssh_command(), Some("ssh -i \"id_rsa\"".into()));
    }

    #[test]
    fn git_ssh_command_without_identity_file() {
        let host = super::Host {
            name: "hodor".into(),
            user: "user".into(),
            build_dir: "build_dir".into(),
            host: None,
            identity_file: None,
        };

        assert_eq!(host.git_ssh_command(), None);
    }

    #[test]
    fn git_ssh_url() {
        let host = super::Host {
            name: "westeros".into(),
            user: "hodor".into(),
            build_dir: "winterfell".into(),
            host: None,
            identity_file: None,
        };

        assert_eq!(host.git_ssh_url(), "hodor@westeros:winterfell");
    }

    #[test]
    fn ssh_command() {
        let host = super::Host {
            name: "westeros".into(),
            user: "hodor".into(),
            build_dir: "winterfell".into(),
            host: Some("the-wall".into()),
            identity_file: Some("id_rsa".into()),
        };

        assert_eq!(host.ssh_command("echo hello"),
                   ["-o",
                    "PreferredAuthentications=publickey",
                    "-i",
                    "id_rsa",
                    "hodor@the-wall",
                    "-C",
                    "echo hello"]);
    }
}
