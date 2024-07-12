use rust_embed::RustEmbed;
use sailfish::TemplateOnce;

#[derive(RustEmbed)]
#[folder = "static/"]
#[exclude = "script.js"]
#[exclude = "input.css"]
pub struct StaticAsset;
#[derive(TemplateOnce)]
#[template(path = "index.stpl")]
pub struct IndexPage {

}