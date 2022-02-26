use crate::compiler::*;
use crate::util::*;
use std::collections::hash_map::HashMap;

pub fn interpret(ast: &Ast, env: &TypeEnv) {
    let mut stack = BucketList::new();

    let mut interp = Interp { env };

    let mut values = HashMap::new();

    let scope = Scope {
        values: &mut values,
        alloc: stack.scoped(),
    };

    interp.block(scope, &ast.block);
}

struct Interp<'a> {
    env: &'a TypeEnv,
}

impl<'a> Interp<'a> {
    fn block(&mut self, mut scope: Scope, block: &Block) {
        for expr in block.stmts {
            self.expr(&mut scope, expr);
        }
    }

    fn expr(&mut self, scope: &mut Scope, e: &Expr) -> Register {
        use ExprKind::*;

        match e.kind {
            Let { value, .. } => {
                let expr = value;
                let value = self.expr(scope, expr);

                scope.values.insert(expr, value);

                return ZERO;
            }

            Ident { .. } => {
                let expr = e as *const Expr;
                let expr = unwrap(self.env.ident_to_expr.get(&expr));
                let register = unwrap(scope.values.get(expr));

                return *register;
            }

            Block(block) => {
                self.block(scope.chain(), &block);

                return ZERO;
            }

            BinaryOp { kind, left, right } => {
                return ZERO;
            }

            _ => unreachable!(),
        }
    }
}

struct Scope<'a> {
    values: &'a mut HashMap<*const Expr, Register>,
    alloc: ScopedBump<'a>,
}

impl<'a> Scope<'a> {
    fn chain<'b>(&'b mut self) -> Scope<'b> {
        return Scope {
            values: self.values,
            alloc: self.alloc.chain(),
        };
    }
}

const ZERO: Register = Register { value: 0 };

#[derive(Clone, Copy)]
struct Register {
    value: u64,
}

impl Register {
    fn u32(&self) -> u32 {
        return self.value as u32;
    }

    fn u64(&self) -> u64 {
        return self.value;
    }
}
