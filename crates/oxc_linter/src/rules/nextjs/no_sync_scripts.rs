use oxc_ast::{
    ast::{JSXAttributeItem, JSXAttributeName, JSXElementName},
    AstKind,
};
use oxc_diagnostics::{
    miette::{self, Diagnostic},
    thiserror::Error,
};
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;
use rustc_hash::FxHashSet;

use crate::{context::LintContext, rule::Rule, AstNode};

#[derive(Debug, Error, Diagnostic)]
#[error("eslint-plugin-next(no-sync-scripts): Prevent synchronous scripts.")]
#[diagnostic(severity(warning), help("See https://nextjs.org/docs/messages/no-sync-scripts"))]
struct NoSyncScriptsDiagnostic(#[label] pub Span);

#[derive(Debug, Default, Clone)]
pub struct NoSyncScripts;

declare_oxc_lint!(
    /// ### What it does
    ///
    ///
    /// ### Why is this bad?
    ///
    ///
    /// ### Example
    /// ```javascript
    /// ```
    NoSyncScripts,
    correctness
);

impl Rule for NoSyncScripts {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::JSXOpeningElement(jsx_opening_element) = node.kind() else { return };

        let JSXElementName::Identifier(jsx_opening_element_name) = &jsx_opening_element.name else {
            return;
        };

        if jsx_opening_element_name.name.as_str() != "script" {
            return;
        }

        let attributes_hs =
            jsx_opening_element
                .attributes
                .iter()
                .filter_map(|v| {
                    if let JSXAttributeItem::Attribute(v) = v {
                        Some(&v.name)
                    } else {
                        None
                    }
                })
                .filter_map(|v| {
                    if let JSXAttributeName::Identifier(v) = v {
                        Some(v.name.clone())
                    } else {
                        None
                    }
                })
                .collect::<FxHashSet<_>>();

        if attributes_hs.contains("src")
            && !attributes_hs.contains("async")
            && !attributes_hs.contains("defer")
        {
            ctx.diagnostic(NoSyncScriptsDiagnostic(jsx_opening_element_name.span));
        }
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        r"import {Head} from 'next/document';
			
			      export class Blah extends Head {
			        render() {
			          return (
			            <div>
			              <h1>Hello title</h1>
			              <script src='https://blah.com' async></script>
			            </div>
			          );
			        }
			    }",
        r"import {Head} from 'next/document';
			
			      export class Blah extends Head {
			        render(props) {
			          return (
			            <div>
			              <h1>Hello title</h1>
			              <script {...props} ></script>
			            </div>
			          );
			        }
			    }",
    ];

    let fail = vec![
        r"
			      import {Head} from 'next/document';
			
			        export class Blah extends Head {
			          render() {
			            return (
			              <div>
			                <h1>Hello title</h1>
			                <script src='https://blah.com'></script>
			              </div>
			            );
			          }
			      }",
        r"
			      import {Head} from 'next/document';
			
			        export class Blah extends Head {
			          render(props) {
			            return (
			              <div>
			                <h1>Hello title</h1>
			                <script src={props.src}></script>
			              </div>
			            );
			          }
			      }",
    ];

    Tester::new_without_config(NoSyncScripts::NAME, pass, fail).test_and_snapshot();
}
