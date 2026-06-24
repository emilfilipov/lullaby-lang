use nous_parser::Program;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IrModule {
    pub function_count: usize,
}

pub fn lower(program: &Program) -> IrModule {
    IrModule {
        function_count: program.functions.len(),
    }
}
