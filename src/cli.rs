pub mod cli {

    use clap::{Parser, Subcommand};

    /// Add, delete, or modify users in LDAP and Slurm simultaneously
    #[derive(Parser, Debug)]
    #[clap(author = "Author: Dominik Wagner", version = "0.1.2", 
            about = "Simultaneous Slurm and LDAP user management", long_about = None)]
    pub struct Args {
        /// Operation to conduct on the user. Either add, delete or modify.
        #[clap(subcommand)]
        pub command: Commands,
        /// Manage the user in Slurm only.
        #[clap(long)]
        pub slurm_only: bool,
        /// Manage the user in LDAP only.
        #[clap(long)]
        pub ldap_only: bool,
    }

    #[derive(Subcommand, Debug)]
    pub enum Commands {
        /// Add a user to Slurm and/or LDAP
        Add { 
            /// Username e.g. wagnerdo.
            user: String,
            /// Unix group the user belongs to e.g. staff.
            #[clap(short, long, default_value = "student")]
            group: String,
            /// Firstname of the user.
            #[clap(short, long)]
            firstname: String,
            /// Lastname of the user.
            #[clap(short, long)]
            lastname: String,
            /// User's e-mail address.
            #[clap(short, long, default_value = "")]
            mail: String,
            /// Slurm default QOS for the user e.g. basic.
            #[clap(short, long, default_value = "basic")]
            default_qos: String,
            /// List of QOS assigned to the user (must be valid QOS i.e. they must exist in valid_qos of conf.toml). 
            #[clap(short, long, max_values(20))]
            qos: Vec<String>,
        },
        /// Modify a user in Slurm and/or LDAP
        Modify { 
            /// A valid username e.g. wagnerdo.
            user: String, 
            /// Firstname of the user.
            #[clap(short, long)]
            firstname: Option<String>,
            /// Lastname of the user.
            #[clap(short, long)]
            lastname: Option<String>,
            /// User's e-mail address.
            #[clap(short, long)]
            mail: Option<String>,
            /// Slurm default QOS for the user e.g. basic.
            #[clap(short, long)]
            default_qos: Option<String>,
            /// List of QOS assigned to the user (must be valid QOS i.e. they must exist in valid_qos of conf.toml). 
            #[clap(short, long)]
            qos: Vec<String>
        },
        /// Delete a user from Slurm and/or LDAP
        Delete { 
            /// A valid username e.g. wagnerdo.
            user: String 
        },
    }
}


