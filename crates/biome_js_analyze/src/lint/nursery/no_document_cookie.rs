use biome_analyze::{context::RuleContext, declare_lint_rule, Rule, RuleDiagnostic, RuleSource};
use biome_console::markup;
use biome_js_semantic::SemanticModel;
use biome_js_syntax::{
    global_identifier, AnyJsAssignment, AnyJsExpression, JsAssignmentExpression,
};
use biome_rowan::AstNode;

use crate::services::semantic::Semantic;

declare_lint_rule! {
    /// Disallow direct assignments to `document.cookie`.
    ///
    /// It's not recommended to use document.cookie directly as it's easy to get the string wrong.
    /// Instead, you should use the [Cookie Store API](https://developer.mozilla.org/en-US/docs/Web/API/CookieStore).
    ///
    /// ## Examples
    ///
    /// ### Invalid
    ///
    /// ```js,expect_diagnostic
    /// document.cookie = "foo=bar";
    /// ```
    ///
    /// ```js,expect_diagnostic
    /// document.cookie += "; foo=bar";
    /// ```
    ///
    /// ### Valid
    ///
    /// ```js
    /// const array = document.cookie.split("; ");
    /// ```
    ///
    /// ```js
    /// await cookieStore
    ///   .set({
    ///     name: "foo",
    ///     value: "bar",
    ///     expires: Date.now() + 24 * 60 * 60,
    ///     domain: "example.com",
    /// })
    /// ```
    ///
    /// ```js
    /// import Cookies from 'js-cookie';
    ///
    /// Cookies.set('foo', 'bar');
    /// ```
    ///
    pub NoDocumentCookie {
        version: "next",
        name: "noDocumentCookie",
        language: "js",
        recommended: false,
        sources: &[RuleSource::EslintUnicorn("no-document-cookie")],
    }
}

/// Check `expr` is `document`
fn is_global_document(expr: &AnyJsExpression, model: &SemanticModel) -> Option<()> {
    let (reference, name) = global_identifier(expr)?;

    // Check identifier is `document`
    if name.text() != "document" {
        return None;
    };

    // TODO: Verify that the variable is assigned the global `document` to be closer to the original rule.
    model.binding(&reference).is_none().then_some(())
}

/// Check member is `cookie`
fn is_cookie(assignment: &AnyJsAssignment) -> Option<()> {
    const COOKIE: &str = "cookie";
    match assignment {
        // `document.cookie`
        AnyJsAssignment::JsStaticMemberAssignment(static_assignment) => {
            let property = static_assignment.member().ok()?;

            if property.text() != COOKIE {
                return None;
            };
        }
        // `document["cookie"]`
        AnyJsAssignment::JsComputedMemberAssignment(computed_assignment) => {
            let any_expr = computed_assignment.member().ok()?;
            let string_literal = any_expr
                .as_any_js_literal_expression()?
                .as_js_string_literal_expression()?;
            let inner_string = string_literal.inner_string_text().ok()?;

            if inner_string.text() != COOKIE {
                return None;
            }
        }
        _ => {
            return None;
        }
    }

    Some(())
}

impl Rule for NoDocumentCookie {
    type Query = Semantic<JsAssignmentExpression>;
    type State = ();
    type Signals = Option<Self::State>;
    type Options = ();

    fn run(ctx: &RuleContext<Self>) -> Self::Signals {
        let node = ctx.query();
        let left = node.left().ok()?;

        let any_assignment = left.as_any_js_assignment()?;

        let expr = match any_assignment {
            AnyJsAssignment::JsStaticMemberAssignment(assignment) => assignment.object().ok()?,
            AnyJsAssignment::JsComputedMemberAssignment(assignment) => assignment.object().ok()?,
            _ => {
                return None;
            }
        };

        is_global_document(&expr, ctx.model())?;

        is_cookie(any_assignment)?;

        Some(())
    }

    fn diagnostic(ctx: &RuleContext<Self>, _state: &Self::State) -> Option<RuleDiagnostic> {
        let node = ctx.query();
        Some(
            RuleDiagnostic::new(
                rule_category!(),
                node.range(),
                markup! {
                    "Direct assigning to "<Emphasis>"document.cookie"</Emphasis>" is not recommended."
                },
            )
            .note(markup! {
                "Consider using the "<Hyperlink href = "https://developer.mozilla.org/en-US/docs/Web/API/CookieStore">"Cookie Store API"</Hyperlink>"."
            }),
        )
    }
}
