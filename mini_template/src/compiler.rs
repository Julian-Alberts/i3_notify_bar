use super::{Statement, StorageMethod};

/// This code is only used in tests
/// TODO clean up
impl PartialEq for Statement {
    fn eq(&self, other: &Statement) -> bool {
        match (self, other) {
            (Statement::Literal(s), Statement::Literal(o)) => unsafe { s.as_ref() == o.as_ref() },
            (
                Statement::Calculated {
                    var_name: s_var_name,
                    modifiers: s_modifiers,
                },
                Statement::Calculated {
                    var_name: o_var_name,
                    modifiers: o_modifiers,
                },
            ) => {
                unsafe {
                    if s_var_name.as_ref().unwrap() != o_var_name.as_ref().unwrap() {
                        return false;
                    }
                }

                if s_modifiers.len() != o_modifiers.len() {
                    return false;
                }

                if s_modifiers.iter().zip(o_modifiers).any(|(sp, op)| {
                    unsafe {
                        if sp.0.as_ref() != op.0.as_ref() {
                            return true;
                        }
                    }

                    if sp.1.iter().zip(op.1.iter()).any(|(s, o)| {
                        match (s, o) {
                            (StorageMethod::Const(s), StorageMethod::Const(o)) => {
                                if s != o {
                                    return true;
                                }
                            }
                            (StorageMethod::Variable(s), StorageMethod::Variable(o)) => unsafe {
                                if s.as_ref() != o.as_ref() {
                                    return true;
                                }
                            },
                            _ => return true,
                        }
                        false
                    }) {
                        return true;
                    }
                    false
                }) {
                    return false;
                }

                true
            }
            _ => false,
        }
    }
}
