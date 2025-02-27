use std::mem::take;

use anyhow::{bail, Error};
use swc_atoms::js_word;
use swc_common::{SyntaxContext, DUMMY_SP};
use swc_ecma_ast::*;
use swc_ecma_utils::{contains_top_level_await, find_pat_ids, private_ident, ExprFactory};
use swc_ecma_visit::{noop_fold_type, Fold};

use crate::{bundler::chunk::merge::Ctx, modules::Modules, Bundler, Load, ModuleId, Resolve};

impl<L, R> Bundler<'_, L, R>
where
    L: Load,
    R: Resolve,
{
    //  Converts
    ///
    /// ```ts
    /// export const arr = [1, 2, 3];
    /// ```
    ///
    /// to
    ///
    /// ```ts
    /// const _mod = (function(){
    ///     const arr = [1, 2, 3];
    ///     return {
    ///         arr,
    ///     };
    /// })();
    /// ```
    pub(super) fn wrap_esm(
        &self,
        ctx: &Ctx,
        id: ModuleId,
        module: Modules,
    ) -> Result<Modules, Error> {
        let span = DUMMY_SP;
        let module_var_name = match self.scope.wrapped_esm_id(id) {
            Some(v) => v,
            None => bail!("{:?} should not be wrapped with a function", id),
        };
        let injected_ctxt = self.injected_ctxt;

        let is_async = module.iter().any(|m| contains_top_level_await(m.1));

        let mut additional_items = vec![];

        module.iter().for_each(|(module_id, item)| {
            match item {
                // Handle `export *`-s from dependency modules.
                //
                // See: https://github.com/denoland/deno/issues/9200
                ModuleItem::ModuleDecl(ModuleDecl::ExportNamed(NamedExport {
                    span,
                    ref specifiers,
                    ..
                })) if span.ctxt == injected_ctxt => {
                    for s in specifiers {
                        if let ExportSpecifier::Named(ExportNamedSpecifier {
                            orig,
                            exported: Some(exported),
                            ..
                        }) = s
                        {
                            let exported = match exported {
                                ModuleExportName::Ident(ident) => ident,
                                ModuleExportName::Str(..) => {
                                    unimplemented!("module string names unimplemented")
                                }
                            };
                            if ctx.transitive_remap.get(&exported.span.ctxt).is_some() {
                                let specifier = ExportSpecifier::Named(ExportNamedSpecifier {
                                    span: DUMMY_SP,
                                    orig: orig.clone(),
                                    exported: Some(ModuleExportName::Ident(exported.clone())),
                                    is_type_only: false,
                                });
                                additional_items.push((
                                    module_id,
                                    ModuleItem::ModuleDecl(ModuleDecl::ExportNamed(NamedExport {
                                        span: DUMMY_SP.with_ctxt(injected_ctxt),
                                        specifiers: vec![specifier],
                                        src: None,
                                        type_only: false,
                                        with: None,
                                    })),
                                ));
                            }
                        }
                    }
                }
                _ => {}
            }
        });

        let mut export_visitor = ExportToReturn {
            synthesized_ctxt: self.synthesized_ctxt,
            return_props: Default::default(),
        };
        let mut module = module.fold_with(&mut export_visitor);

        module.append_all(additional_items);

        let return_stmt = Stmt::Return(ReturnStmt {
            span: DUMMY_SP,
            arg: Some(Box::new(Expr::Object(ObjectLit {
                span: DUMMY_SP,
                props: take(&mut export_visitor.return_props),
            }))),
        });

        module.iter().for_each(|(_, v)| {
            if let ModuleItem::ModuleDecl(ModuleDecl::ExportAll(ref export)) = v {
                // We handle this later.
                let mut map = ctx.export_stars_in_wrapped.lock();
                map.entry(id).or_default().push(export.span.ctxt);
            }
        });

        let module_fn = Expr::Fn(FnExpr {
            function: Box::new(Function {
                params: Default::default(),
                decorators: Default::default(),
                span: DUMMY_SP,
                body: Some(BlockStmt {
                    span: DUMMY_SP,
                    stmts: vec![return_stmt],
                }),
                is_generator: false,
                is_async,
                type_params: Default::default(),
                return_type: Default::default(),
            }),
            ident: None,
        });

        let mut module_expr = Expr::Call(CallExpr {
            span: DUMMY_SP,
            callee: module_fn.as_callee(),
            type_args: Default::default(),
            args: Default::default(),
        });

        if is_async {
            module_expr = Expr::Await(AwaitExpr {
                span: DUMMY_SP,
                arg: Box::new(module_expr),
            });
        }

        let var_decl = VarDecl {
            span: span.with_ctxt(self.injected_ctxt),
            declare: false,
            kind: VarDeclKind::Const,
            decls: vec![VarDeclarator {
                span: DUMMY_SP,
                definite: false,
                name: Pat::Ident(module_var_name.into_ident().into()),
                init: Some(Box::new(module_expr)),
            }],
        };

        module.append(
            id,
            ModuleItem::Stmt(Stmt::Decl(Decl::Var(Box::new(var_decl)))),
        );

        // print_hygiene(
        //     "wrap",
        //     &self.cm,
        //     &Module {
        //         span: DUMMY_SP,
        //         body: module_items.clone(),
        //         shebang: None,
        //     },
        // );

        Ok(module)
    }
}

struct ExportToReturn {
    return_props: Vec<PropOrSpread>,
    synthesized_ctxt: SyntaxContext,
}

impl ExportToReturn {
    fn export_id(&mut self, mut i: Ident) {
        i.span.ctxt = SyntaxContext::empty();
        self.return_props
            .push(PropOrSpread::Prop(Box::new(Prop::Shorthand(i))));
    }

    fn export_key_value(&mut self, mut key: Ident, value: Ident) {
        key.span.ctxt = SyntaxContext::empty();

        self.return_props
            .push(PropOrSpread::Prop(Box::new(Prop::KeyValue(KeyValueProp {
                key: PropName::Ident(key),
                value: Box::new(Expr::Ident(value)),
            }))));
    }
}

impl Fold for ExportToReturn {
    noop_fold_type!();

    fn fold_stmt(&mut self, s: Stmt) -> Stmt {
        s
    }

    fn fold_module_item(&mut self, item: ModuleItem) -> ModuleItem {
        let decl = match item {
            ModuleItem::ModuleDecl(decl) => decl,
            ModuleItem::Stmt(_) => return item,
        };

        let stmt = match decl {
            ModuleDecl::Import(_) => return ModuleItem::ModuleDecl(decl),
            ModuleDecl::ExportDecl(export) => {
                match &export.decl {
                    Decl::Class(ClassDecl { ident, .. }) | Decl::Fn(FnDecl { ident, .. }) => {
                        self.export_id(ident.clone());
                    }
                    Decl::Var(decl) => {
                        let ids: Vec<Ident> = find_pat_ids(decl);
                        ids.into_iter().for_each(|id| self.export_id(id));
                    }
                    _ => unreachable!(),
                }

                Some(Stmt::Decl(export.decl))
            }

            ModuleDecl::ExportDefaultDecl(export) => match export.decl {
                DefaultDecl::Class(expr) => {
                    let ident = expr.ident;
                    let ident = ident.unwrap_or_else(|| private_ident!("_default_decl"));

                    self.export_key_value(
                        Ident::new(js_word!("default"), export.span),
                        ident.clone(),
                    );

                    Some(Stmt::Decl(Decl::Class(ClassDecl {
                        ident,
                        class: expr.class,
                        declare: false,
                    })))
                }
                DefaultDecl::Fn(expr) => {
                    let ident = expr.ident;
                    let ident = ident.unwrap_or_else(|| private_ident!("_default_decl"));

                    self.export_key_value(
                        Ident::new(js_word!("default"), export.span),
                        ident.clone(),
                    );

                    Some(Stmt::Decl(Decl::Fn(FnDecl {
                        ident,
                        function: expr.function,
                        declare: false,
                    })))
                }
                DefaultDecl::TsInterfaceDecl(_) => None,
            },
            ModuleDecl::ExportDefaultExpr(_) => None,
            ModuleDecl::ExportAll(export) => {
                return ModuleItem::ModuleDecl(ModuleDecl::ExportAll(export))
            }
            ModuleDecl::ExportNamed(export) => {
                for specifier in &export.specifiers {
                    match specifier {
                        ExportSpecifier::Namespace(_) => {}
                        ExportSpecifier::Default(_) => {}
                        ExportSpecifier::Named(named) => match &named.exported {
                            Some(ModuleExportName::Ident(exported)) => {
                                // As injected named exports are converted to variables by other
                                // passes, we should not create a variable for it.
                                if let ModuleExportName::Ident(orig) = &named.orig {
                                    self.export_key_value(exported.clone(), orig.clone());
                                } else {
                                    unimplemented!("module string names unimplemented")
                                }
                            }
                            Some(ModuleExportName::Str(..)) => {
                                unimplemented!("module string names unimplemented")
                            }
                            None => {
                                if let ModuleExportName::Ident(orig) = &named.orig {
                                    self.export_id(orig.clone());
                                } else {
                                    unimplemented!("module string names unimplemented")
                                }
                            }
                        },
                    }
                }

                // Ignore export {} specified by user.
                if export.src.is_none() && export.span.ctxt != self.synthesized_ctxt {
                    None
                } else {
                    return ModuleItem::ModuleDecl(ModuleDecl::ExportNamed(export));
                }
            }
            ModuleDecl::TsImportEquals(_) => None,
            ModuleDecl::TsExportAssignment(_) => None,
            ModuleDecl::TsNamespaceExport(_) => None,
        };

        if let Some(stmt) = stmt {
            ModuleItem::Stmt(stmt)
        } else {
            ModuleItem::Stmt(Stmt::Empty(EmptyStmt { span: DUMMY_SP }))
        }
    }
}
