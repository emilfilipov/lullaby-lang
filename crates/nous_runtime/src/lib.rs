use nous_parser::Program;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimePlan {
    pub entry_function: Option<String>,
}

pub fn plan(program: &Program) -> RuntimePlan {
    RuntimePlan {
        entry_function: program
            .functions
            .iter()
            .find(|function| function.name == "main")
            .map(|function| function.name.clone()),
    }
}
