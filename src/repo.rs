// Definition of repo-managing stuff

use git2::Repository;
use log::{debug, info};
use std::fs;
use std::fs::File;
use std::io::{Error, ErrorKind, Write};
use std::path::Path;

use crate::Credential;

#[derive(Debug, Default)]
pub struct CredentialRepository {
    pub path: String,
}

impl CredentialRepository {
    fn check_repo(&self) -> Result<(), Error> {
        debug!("Checking repo at: {}", self.path);
        if !Path::new(&self.path).is_dir() {
            return Err(Error::new(
                ErrorKind::NotFound,
                format!(
                    "Path '{}' does not exist. Try using 'init' command.",
                    &self.path
                ),
            ));
        }
        Ok(())
    }

    pub fn init(&self) -> Result<(), Error> {
        match self.check_repo() {
            Ok(_) => {
                return Err(Error::new(
                    ErrorKind::AlreadyExists,
                    format!("Path '{}' already exists.", &self.path),
                ));
            }
            Err(_) => {
                info!("Repo doesn't exist. Creating.");
                // First, create repo directory
                fs::create_dir_all(&self.path)?;
                // Then initialise it as a git repo
                match Repository::init(&self.path) {
                    Ok(_) => {}
                    Err(e) => {
                        return Err(Error::new(
                            ErrorKind::Other,
                            format!("Failed to create repo at '{}': {}", &self.path, e),
                        ));
                    }
                }
                // Create a subdirectory for credentials
                let creds_dir = self.path.clone() + "/credentials";
                fs::create_dir_all(creds_dir)?;
            }
        }
        Ok(())
    }

    fn list_creds(&self, dir: &str, grp: &str) -> Result<(), Error> {
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let metadata = fs::metadata(entry.path())?;
            if metadata.is_file() {
                // Output credential. TODO: Also include username or whatever in output?
                println!("{}/{}", grp, entry.file_name().to_string_lossy());
            } else if metadata.is_dir() {
                // new group
                let newgrp = format!("{}/{}", grp, entry.file_name().to_string_lossy());
                // recurse
                self.list_creds(&entry.path().into_os_string().to_string_lossy(), &newgrp)?;
            }
        }
        Ok(())
    }

    pub fn list(&self) -> Result<(), Error> {
        // If repo doesn't exist, bail.
        self.check_repo()?;

        // Each group is a directory and each file is a credential
        let dir = self.path.clone() + "/credentials";
        // recurse
        self.list_creds(&dir, "")
    }

    pub fn get(&self, name: &str) -> Result<Credential, Error> {
        self.check_repo()?;
        info!(
            "Looking for credential {} at {}/credentials/{}.cred",
            name, self.path, name
        );
        let path = format!("{}/credentials/{}.cred", self.path, name);
        Credential::from_file(&path)
    }

    pub fn set(&self, name: &str, username: &str, password: &str, key: &str) -> Result<(), Error> {
        self.check_repo()?;
        info!("Setting credential {}", name);
        let cred = Credential::from_input(name, username, password, key);
        let cred = cred.unwrap();
        info!("Credential: {:?}", cred.as_json());

        // Write the JSON object out to a file
        let f = self.path.clone() + "/credentials/" + name + ".cred";
        info!("Writing to {}", f);
        let mut outfile = File::create(f).unwrap();
        write!(outfile, "{}", cred.as_json().unwrap()).unwrap();

        Ok(())
    }
}
