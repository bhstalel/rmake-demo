/// This represents the rmake utilities
pub mod rmake {
    use crate::RMakeError;
    use regex::Regex;
    use serde_yaml::{Mapping, Value};
    use std::process::Command;
    use std::{collections::HashMap, vec};
    use tracing::{debug, error, info};

    /// This represents a Core command that can be run
    pub enum RMakeCoreCommand {
        /// A shell command
        Shell,

        /// A wildcard command
        Wildcard,
    }

    /// Implementation of FromStr
    ///
    /// This returns a RMakeCoreCommand from a given str
    ///
    /// # Arguments:
    ///
    /// * s - The given str
    impl std::str::FromStr for RMakeCoreCommand {
        type Err = String;
        fn from_str(s: &str) -> Result<Self, Self::Err> {
            match s {
                "shell" => Ok(Self::Shell),
                "whildcard" => Ok(Self::Wildcard),
                &_ => Err(format!("{} Is not supported yet!", s)),
            }
        }
    }

    /// This represents a Dependency
    #[derive(Debug)]
    pub enum _RMakeDependency {
        /// The dep is a File that needs to check its modified date
        _File(String),

        /// The dep is another target
        _Target(RMakeTarget),
    }

    /// This represents a Target
    #[derive(Debug, Clone)]
    pub struct RMakeTarget {
        /// The name of the target
        pub name: String,

        /// The list of dependencies, this is Option in case of no deps
        pub deps: Option<Vec<String>>,

        /// The list of commands that needs to be run on the target visit
        pub cmds: Vec<String>,
    }

    /// This represents a Variable
    #[derive(Debug)]
    pub struct RMakeVariable {
        /// The name of the variable, issued from a String YAML Value
        pub name: String,

        /// The value of the variable, MUST be a String, though YAML supports more
        pub value: String,
    }

    impl RMakeVariable {
        /// Construct an RMakeVariable from a given YAML Value
        ///
        /// # Arguments:
        ///
        /// * name - The name of the variable
        /// * value - The YAML Value object
        ///
        /// Returns an Option indicating the Value is string or not
        pub fn from_value(name: String, value: &Value) -> Option<RMakeVariable> {
            if value.is_string() {
                return Some(RMakeVariable {
                    name: name,
                    value: value.as_str().unwrap().to_string(),
                });
            }
            None
        }
    }

    /// Defining custom types
    type RMakeTargets = HashMap<String, RMakeTarget>;
    type RMakeVariables = HashMap<String, RMakeVariable>;

    /// This represents the main object of RMake project
    #[derive(Debug)]
    pub struct RMake {
        /// List of targets of the YAML file
        pub targets: RMakeTargets,

        /// List of variables of the YAML file, this is Option because you can have no variables
        pub variables: Option<RMakeVariables>,
    }

    impl RMake {
        /// Extract all Mappings and variables in the global Mapping
        ///
        /// # Arguments:
        ///
        /// * global_map - The global mapping for the YAML file
        ///
        /// Returns a tuple of two Option of HashMaps for Targets and Variables
        fn extract_targets_and_variables(
            global_map: &Mapping,
        ) -> (Option<RMakeTargets>, Option<RMakeVariables>) {
            let mut inner_targets = HashMap::new();
            let mut inner_variables = HashMap::new();

            for (key, val) in global_map {
                let key_name = key.as_str().unwrap().to_string();
                if val.is_mapping() {
                    inner_targets.insert(
                        key_name.clone(),
                        RMakeTarget::from_mapping(key_name, val.as_mapping().unwrap()),
                    );
                } else {
                    let var_value = RMakeVariable::from_value(key_name.clone(), val);
                    if var_value.is_some() {
                        inner_variables.insert(key_name.clone(), var_value.unwrap());
                    }
                }
            }

            (
                if inner_targets.len() > 0 {
                    Some(inner_targets)
                } else {
                    None
                },
                if inner_variables.len() > 0 {
                    Some(inner_variables)
                } else {
                    None
                },
            )
        }

        /// Load file content and extract all variables and targets
        ///
        /// # Arguments:
        ///
        /// * path - The RMakefile.yml path
        ///
        /// Returns a Result Self object
        pub fn new(path: String) -> Result<RMake, ()> {
            if let Ok(yml_c) = RMake::load_yml(path) {
                /* Content MUST be Mapping */
                if !yml_c.is_mapping() {
                    panic!("The Yml file should be Mapping, check the format!");
                }

                /* We are sure that this is Mapping, so unwrap is safe here !*/
                let mapping = yml_c.as_mapping().unwrap();

                /* Extract all Mappings and Variables */
                let (targets, variables) = RMake::extract_targets_and_variables(mapping);

                if targets.is_none() {
                    panic!("No target is defined in the input file!");
                }

                let mut targets = targets.unwrap();

                /* Expand commands */
                for (name, mut target_obj) in targets.clone().into_iter() {
                    target_obj.expand_commands(&variables);
                    *targets.get_mut(&name).unwrap() = target_obj.clone();
                }

                return Ok(RMake {
                    targets: targets,
                    variables: variables,
                });
            }

            Err(())
        }

        /// Load YAML content from a given file
        ///
        /// # Arguments:
        ///
        /// * path - The file path
        ///
        /// Returns content String or Error on failure.
        fn load_yml(path: String) -> Result<serde_yaml::Value, Box<dyn std::error::Error>> {
            let reader = std::fs::File::open(path)?;
            match serde_yaml::from_reader(reader) {
                Ok(yml) => Ok(yml),
                Err(e) => Err(Box::new(e)),
            }
        }

        #[allow(unused)]
        fn get_target(&self, name: String) -> Option<&RMakeTarget> {
            self.targets.get(&name)
        }

        #[allow(unused)]
        fn count_deps(&self) -> usize {
            let mut sum = 0;
            for (_, target) in self.targets.iter() {
                if let Some(deps) = &target.deps {
                    sum += deps.len();
                }
            }
            sum
        }

        /// Chain all commands of all targets in order
        ///
        /// # Arguments:
        ///
        /// * main_target - The starting target
        ///
        /// Returns a Vector of String
        pub fn chain_commands(&mut self, main_target: RMakeTarget) -> Vec<String> {
            /// Inner function to use it in recursive mode
            ///
            /// # Arguments:
            ///
            /// * target - The RMakeTarget to continue with
            /// * targets - All RMakeTargets will be used to look for dependencies
            /// * visited - A bool HashMap to mark that a Target is visited/found or not
            ///
            /// Returns a Vector of String that will accumulated recursively
            fn find(
                target: &RMakeTarget,
                targets: &RMakeTargets,
                visited: &mut HashMap<String, bool>,
            ) -> Vec<String> {
                let mut ret_command = vec![];

                if let Some(dependencies) = &target.deps {
                    for dep in dependencies {
                        if let Some(sub_target) = targets.get(dep) {
                            if !visited.contains_key(dep) {
                                visited.insert(dep.clone(), true);
                                ret_command.extend(find(sub_target, targets, visited))
                            }
                        }
                    }
                }

                ret_command.extend(target.cmds.clone());
                ret_command
            }

            let mut visited = HashMap::new();
            let command_chain = find(&main_target, &self.targets, &mut visited);
            command_chain
        }

        /// Run the RMake system
        ///
        /// # Arguments:
        ///
        /// * name - The target name
        pub fn run(&mut self, name: String) {
            if let Some(main_target) = self.targets.get(&name) {
                for cmd in self.chain_commands(main_target.clone()) {
                    info!("Running: {}", cmd);
                }
            } else {
                RMakeError!("No rule to make target: {}", name);
            }
        }
    }

    impl RMakeTarget {
        #[allow(unused)]
        pub fn from_global(name: String, mapping: &Mapping) -> RMakeTarget {
            if !mapping.contains_key(name.clone()) {
                RMakeError!("Target {} not found in YAML file!", name);
            }

            match mapping.get(name.clone()).unwrap() {
                Value::Mapping(target_map) => RMakeTarget::from_mapping(name, target_map),
                _ => {
                    RMakeError!("Target type is not Mapping!");
                }
            }
        }

        /// Create RMakeTarget from a YAML Mapping object
        ///
        /// # Arguments:
        ///
        /// * name - The name of the target
        /// * mapping - The Mapping object
        pub fn from_mapping(name: String, mapping: &Mapping) -> RMakeTarget {
            if !mapping.contains_key("cmd") {
                RMakeError!("A target must have cmd field!");
            }

            /* Construct dependencies names */
            let mut deps_strings: Vec<String> = vec![];
            if mapping.contains_key("dep") {
                let deps = mapping.get("dep").unwrap();
                if deps.is_string() {
                    deps_strings.push(deps.as_str().unwrap().to_string());
                } else if deps.is_sequence() {
                    for v in deps.as_sequence().unwrap() {
                        if v.is_string() {
                            deps_strings.push(v.as_str().unwrap().to_string());
                        }
                    }
                }
            }

            /*
             *   Construct commands list
             *   It can be:
             *       "cmd": ["cmd1", "cmd2"]
             *       => will be parsed to: Sequence(String)
             *   or
             *       "cmd": |
             *           cmd1
             *           cmd2
             *       => will be parsed to: String("cmd1\ncmd2")
             */
            let mut cmds_list: Vec<String> = vec![];

            let cmds = mapping.get("cmd").unwrap();
            match cmds.as_str() {
                Some(s_content) => {
                    /* Split the conent by \n */
                    for s in s_content.split("\n") {
                        cmds_list.push(s.to_string());
                    }
                }
                None => {
                    /* Try parsing as Sequence */
                    match cmds.as_sequence() {
                        Some(seq_content) => {
                            for seq_elem in seq_content {
                                match seq_elem.as_str() {
                                    Some(cmd) => cmds_list.push(cmd.to_string()),
                                    None => {
                                        RMakeError!("Command in the Sequence is not String");
                                    }
                                }
                            }
                        }
                        None => {
                            /* Format is neither Sequence nor String */
                            RMakeError!("Command list is not Sequence nor String !");
                        }
                    }
                }
            }

            let ret_deps = if deps_strings.len() > 0 {
                Some(deps_strings)
            } else {
                None
            };

            RMakeTarget {
                name: name,
                deps: ret_deps,
                cmds: cmds_list,
            }
        }

        /// Loop through all commands and expand them
        ///
        /// # Arguments:
        ///
        /// * variables - Optional list of all variables of the YAML file
        fn expand_commands(&mut self, variables: &Option<RMakeVariables>) {
            let mut final_commands = vec![];
            for command in self.cmds.clone().into_iter() {
                debug!("Expanding command: {}", command);
                let cmd = RMakeUtils::find_and_replace(
                    command,
                    RMakeUtils::default_rmake_regex(),
                    variables,
                );
                final_commands.push(cmd.clone());
                debug!(" --------------- \n");
            }
            self.cmds = final_commands;
        }
    }

    #[allow(non_snake_case)]
    mod RMakeUtils {

        use super::{RMakeCoreCommand, RMakeVariables};
        use crate::RMakeError;
        use regex::Regex;
        use std::process::Command;
        use std::str::FromStr;
        use tracing::{debug, error, warn};
        use tracing_subscriber::field::debug;

        pub fn default_rmake_regex() -> Regex {
            Regex::new(r"\$\(([^)]+)\)").unwrap()
        }

        /// Find a regex and replace it in all the given String
        ///
        /// # Arguments:
        ///
        /// * value - The full String input
        /// * re - The Regex
        /// * variables - The full RMake variable list
        ///
        /// Returns the processed String input
        pub fn find_and_replace(
            value: String,
            re: regex::Regex,
            variables: &Option<RMakeVariables>,
        ) -> String {
            let mut value = value;
            for found in re.find_iter(&value.clone()) {
                /* If variable does not exist, ignoring by default */
                let mut to = String::from("");

                /* Get variable value and then expand */
                let found_str = found.as_str();
                let found_str = &found_str[2..found_str.len() - 1];
                let found_str_elems = found_str.split_whitespace().collect::<Vec<_>>();

                debug!(
                    "Found match: {} with elems: {:?}",
                    found_str, found_str_elems
                );

                if found_str_elems.len() == 1 {
                    /* A local variable, check if exist, else, check if it is env variable */
                    let mut check_env = true;
                    if let Some(vars) = variables {
                        if let Some(value) = vars.get(found_str_elems[0]) {
                            /* Expand the variable */
                            debug!(
                                "Expanding variable {} with value: {}",
                                value.name, value.value
                            );
                            to = find_and_replace(
                                value.value.clone(),
                                default_rmake_regex(),
                                variables,
                            );
                            debug!("Expanded variable: {}", to);
                            check_env = false;
                        } else {
                            warn!(
                                "Variable {} is not found in variables, checking env ..",
                                found_str_elems[0]
                            );
                        }
                    };

                    if check_env {
                        if let Ok(env_val) = std::env::var(found_str_elems[0]) {
                            to = env_val;
                            debug!("Found variable value in env: {}", to);
                        }
                    }
                } else if found_str_elems.len() > 1 {
                    debug!("Variable has more than element, cheking RMakeCoreCommands ..");

                    /* This is an RMakeCoreCommand */
                    match RMakeCoreCommand::from_str(found_str_elems[0]) {
                        Ok(core_cmd) => match core_cmd {
                            RMakeCoreCommand::Shell => {
                                /* Run a Shell command and set (to) */
                                let mut shell_command = Command::new(found_str_elems[1]);

                                for i in 2..found_str_elems.len() - 1 {
                                    shell_command.arg(found_str_elems[i]);
                                }

                                to = String::from_utf8(
                                    shell_command
                                        .output()
                                        .expect("Cannot execute command!")
                                        .stdout,
                                )
                                .unwrap();
                            }
                            RMakeCoreCommand::Wildcard => {
                                warn!("wildcard is not yet supported!")
                            }
                        },
                        Err(e) => {
                            RMakeError!("Variable error: {}", e);
                        }
                    }
                }

                debug!("String Before: {}", value);
                value = re.replace(&value, to).to_string();
                debug!("String After: {}", value);
            }

            value
        }
    }
}
