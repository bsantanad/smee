#![allow(non_camel_case_types)]
/*
 *  smee
 *  ----
 *
 *  CLI tool that allows you to manage your kubeconfig files, listing, adding,
 *  unsetting and setting which one should [kubectl][1] use.
 *
 * TODO split code into functions
 * TODO add a bit more docs to the functions
 */
use colored::Colorize;
use clap::{command, Parser, Subcommand};


#[derive(Parser)]
struct cli_s {
    #[command(subcommand)]
    command: commands_e,

    #[command(flatten)]
    verbose: clap_verbosity_flag::Verbosity,

    #[arg(default_value = "~/.kube/")]
    dotkube_path: String,
}

#[derive(Subcommand)]
enum commands_e {
   #[command(about = "List your kubeconfig files under ~/.kube/")]
   ls(list_s),
   #[command(about = "Get current kubeconfig file")]
   current(current_s),
   #[command(about = "Set a kubeconfig file to be used")]
   set(set_s),
   #[command(about = "Unset the current kubeconfig file")]
   unset(unset_s),
   #[command(about = "Add a kubeconfig file to ~/.kube/ dir")]
   add(add_s),
   #[command(about = "Delete a kubeconfig file from your ~/.kube/ dir")]
   delete(delete_s),
}

#[derive(Parser)]
struct list_s { }

#[derive(Parser)]
struct current_s { }

#[derive(Parser)]
struct set_s {
    kubeconfig_file: String,
}

#[derive(Parser)]
struct unset_s { }

#[derive(Parser)]
struct add_s {
    filename: String,
}

#[derive(Parser)]
struct delete_s {
    filename: String,
}


fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = cli_s::parse();

    // init logging with verbosity level sent by user
    env_logger::Builder::new()
        .filter_level(args.verbose.log_level_filter())
        .init();

    let dotkube_path_expanded = shellexpand::tilde(&args.dotkube_path);
    let dotkube_path = std::path::Path::new(dotkube_path_expanded.as_ref());

    match &args.command {

        commands_e::ls(_) => {
            for entry in std::fs::read_dir(&dotkube_path)? {
                let entry = entry?;
                let path = entry.path();
                if path.is_dir() {
                    // TODO: crawl the dir to find more config files
                    continue
                }
                if entry.file_type()?.is_symlink() {
                    let path = std::fs::read_link(path)?;
                    println!(
                        "{} -> {}",
                        entry.file_name().to_string_lossy().as_ref().blue(),
                        path.to_string_lossy().as_ref().blue(),
                    );

                    continue
                }
                // file_name returns OsStr which does not have display()
                // function, so we do string_lossy and then since it returns
                // Cow<_, str> we have to get it with as_ref()
                println!("{}", entry.file_name().to_string_lossy().as_ref())
            }
        }

        commands_e::current(_) => {
            let kubeconfig_path = dotkube_path.join("config");
            if !std::path::Path::new(&kubeconfig_path).exists(){
                println!("kubeconfig file not set");
                log::info!(
                    "There is no current config file under {}, consider \
                      setting it via `set` subcommand. --help for more info",
                    kubeconfig_path.display(),
                );
                return Ok(());
            }

            let path = std::fs::read_link(kubeconfig_path)?;
            println!("{} -> {}",
                     "config".blue(),
                     path.to_string_lossy().as_ref().blue())
        }

        commands_e::set(cmd) => {
            let new_config_fullpath = dotkube_path.join(&cmd.kubeconfig_file);
            if !std::path::Path::new(&new_config_fullpath).exists(){
                println!(
                    "{} does not exists inside {}, consider adding it via \
                     `add` subcommand. Do --help for more info. Bailing out.",
                    cmd.kubeconfig_file.bold(),
                    dotkube_path.display(),
                );
                return Ok(());
            }

            let kubeconfig_file_path = dotkube_path.join("config");
            let _ = match std::fs::remove_file(&kubeconfig_file_path) {
                Ok(()) => "",
                Err(e) if e.kind() == std::io::ErrorKind::NotFound => "",
                Err(e) => panic!("{}", e),
            };

            std::os::unix::fs::symlink(&new_config_fullpath, kubeconfig_file_path)?
        }

        commands_e::unset(_) => {
            let kubeconfig_file_path = dotkube_path.join("config");
            if !kubeconfig_file_path.exists(){
                println!("No kubeconfig file set");
                return Ok(());
            }
            std::fs::remove_file(&kubeconfig_file_path)?;
        }

        commands_e::add(cmd) => {
            let filepath = std::path::Path::new(&cmd.filename);
            if !filepath.exists(){
                println!(
                    "Could not find file: {}. Does it exists? Bailing out",
                    cmd.filename.bold(),
                );
                return Ok(());
            }
            let filename = filepath.file_name()
                .expect("Expected a valid file name");
            let new_path = dotkube_path.join(filename);
            std::fs::copy(&cmd.filename, new_path)?;
        }

        commands_e::delete(cmd) => {
            let new_config_fullpath = dotkube_path.join(&cmd.filename);
            if !std::path::Path::new(&new_config_fullpath).exists(){
                println!(
                    "{} does not exists inside {}, nothing to delete, \
                    bailing out",
                    cmd.filename.bold(),
                    dotkube_path.display(),
                );
                return Ok(())
            }
            std::fs::remove_file(&new_config_fullpath)?;
        }

    }

    Ok(())
}
