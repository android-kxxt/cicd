//! Format the change log into various formats using handlebars

use handlebars::{
    Context, Handlebars, Helper, HelperResult, JsonRender, Output, RenderContext, RenderError,
    RenderErrorReason, handlebars_helper,
};

use crate::changelog::ChangeLog;

/// Escape content to be used in a markdown link description.
///
/// Note: this function is not security oriented. We assume that
///       some input might already be markdown. For example, some
///       may use `code` in commit title.
fn md_link_desc_escape(
    h: &Helper,
    bar: &Handlebars,
    _: &Context,
    rc: &mut RenderContext,
    out: &mut dyn Output,
) -> HelperResult {
    let param = h
        .param(0)
        .ok_or_else(|| RenderErrorReason::ParamNotFoundForIndex("md_link_desc_escape", 0))?;
    let mut s = param.value().render();
    s = s.replace("[", "\\[");
    s = s.replace("]", "\\]");
    s = s.replace("\n", " ");
    let escaped = bar.get_escape_fn()(&s);
    out.write(&escaped)?;
    Ok(())
}

fn indent(
    h: &Helper,
    _: &Handlebars,
    _: &Context,
    rc: &mut RenderContext,
    out: &mut dyn Output,
) -> HelperResult {
    let n = h
        .param(0)
        .ok_or_else(|| RenderErrorReason::ParamNotFoundForIndex("indent", 0))?
        .try_get_constant_value()
        .ok_or_else(|| RenderErrorReason::InvalidParamType("expected a constant integer"))?
        .as_u64()
        .ok_or_else(|| RenderErrorReason::InvalidParamType("expected a constant integer"))?
        .clamp(0, 256);
    let content = h
        .param(1)
        .ok_or_else(|| RenderErrorReason::ParamNotFoundForIndex("indent", 1))?
        .render();
    let whitespace = " ".repeat(n as usize);
    for line in content.lines() {
        out.write(&whitespace)?;
        out.write(&line)?;
        out.write("\n")?;
    }
    Ok(())
}

pub fn format_changelog(template: String, changelog: &ChangeLog) -> Result<String, RenderError> {
    let mut reg = Handlebars::new();
    reg.register_helper("md_link_desc_escape", Box::new(md_link_desc_escape));
    reg.register_helper("indent", Box::new(indent));
    reg.render_template(&template, &changelog)
}
