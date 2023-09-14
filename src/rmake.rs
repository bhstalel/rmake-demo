/// This represents the rmake utilities
pub mod rmake {
    use std::{collections::HashMap, vec};

    use serde_yaml::{Mapping, Value};

    /// This represents a Dependency
    #[derive(Debug)]
    pub enum _RMakeDependency {
        _File(String),
        _Target(RMakeTarget),
    }

    /// This represents a Target
    #[derive(Debug, Clone)]
    pub struct RMakeTarget {
        pub name: String,
        pub deps: Option<Vec<String>>,
        pub cmds: Vec<String>,
    }

    /// This represents a Variable
    #[derive(Debug)]
    pub struct RMakeVariable {
        pub name: String,
        pub value: String,
    }

    impl RMakeVariable {
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

    type RMakeTargets = HashMap<String, RMakeTarget>;
    type RMakeVariables = HashMap<String, RMakeVariable>;

    #[derive(Debug)]
    pub struct RMake {
        pub targets: RMakeTargets,
        pub variables: Option<RMakeVariables>,
    }

    impl RMake {
        /// Extract all Mappings and variables in the global Mapping
        ///
        /// # Arguments:
        ///
        /// * global_map - The global mapping for the YAML file
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
        /// Returns a Self object
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

                return Ok(RMake {
                    targets: targets.unwrap(),
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

        pub fn chain_commands(&mut self, main_target: RMakeTarget) -> Vec<String> {
            fn dfs(
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
                                ret_command.extend(dfs(sub_target, targets, visited))
                            }
                        }
                    }
                }

                ret_command.extend(target.cmds.clone());
                ret_command
            }

            let mut visited = HashMap::new();
            let command_chain = dfs(&main_target, &self.targets, &mut visited);
            command_chain
        }

        pub fn run(&mut self, name: String) {
            if let Some(main_target) = self.targets.get(&name) {
                for cmd in self.chain_commands(main_target.clone()) {
                    println!("Running: {}", cmd);
                }
            } else {
                panic!("No rule to make target: {}", name);
            }
        }
    }

    impl RMakeTarget {
        #[allow(unused)]
        pub fn from_global(name: String, mapping: &Mapping) -> RMakeTarget {
            if !mapping.contains_key(name.clone()) {
                panic!("Target {} not found in YAML file!", name);
            }

            match mapping.get(name.clone()).unwrap() {
                Value::Mapping(target_map) => RMakeTarget::from_mapping(name, target_map),
                _ => panic!("Target type is not Mapping!"),
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
                panic!("A target must have cmd field!");
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
                Construct commands list
                It can be:
                    "cmd": ["cmd1", "cmd2"]
                    => will be parsed to: Sequence(String)
                or
                    "cmd": |
                        cmd1
                        cmd2
                    => will be parsed to: String("cmd1\ncmd2")
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
                                        panic!("Command in the Sequence is not String")
                                    }
                                }
                            }
                        }
                        None => {
                            /* Format is neither Sequence nor String */
                            panic!("Command list is not Sequence nor String !");
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
    }
}
