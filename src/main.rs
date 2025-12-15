// src/main.rs
use std::fs;
use std::path::PathBuf;
use std::path::Path;
use std::env;

use onenote_parser::Parser; // la crate/lib que vous venez de construire (0.3.1)
use onenote_parser::page::PageContent; // la crate/lib que vous venez de construire (0.3.1)

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let input = env::args().nth(1)
        .expect("Usage: one2html_cli <chemin .one> [dossier sortie]");
    let output = env::args().nth(2).unwrap_or_else(|| "out_html".into());

    fs::create_dir_all(&output)?;

    let mut parser = Parser::new();
    // selon la version, l’API peut s’appeler différemment (parse_section / parse_file, etc.)
    // E308 let section = parser.parse_section(PathBuf::from(&input))?;
    let section = parser.parse_section(Path::new(&input))?;

    for series in section.page_series() {
        for (idx, page) in series.pages().iter().enumerate() {
            let default_title = format!("page_{idx}");
            let title : &str = page.title_text().unwrap_or(&default_title);
            let fname = sanitize(title);
            let mut html = String::new();
            html.push_str("<!doctype html><meta charset=\"utf-8\"><title>");
            html.push_str(title);
            html.push_str("</title><body>\n");

            for item in page.contents() {
                use onenote_parser::contents::Content;
                match item {
                    PageContent::Outline(outline) => {
					for it in outline.items() {
						if let Some(elem) = it.element() {
							for oc in elem.contents() {
								match oc {
									Content::RichText(rt) => {
										html.push_str("<p>");
										html.push_str(&escape(rt.text()));
										html.push_str("</p>\n");
									}
									Content::Image(img) => {
										if let Some(bytes) = img.data() {
											let ext = img.extension().unwrap_or("bin");
											let asset = format!("{}/{}_{}.{}", &output, &fname, "img", ext);
											std::fs::write(&asset, bytes)?;
											html.push_str(&format!("<img src=\"{}\" alt=\"image\" />\n", asset));
										}
									}
									Content::EmbeddedFile(file) => {
										let asset = format!("{}/{}_{}", &output, &fname, file.filename());
										std::fs::write(&asset, file.data())?;
										html.push_str(&format!(
											"<a href=\"{}\">{}</a>\n",
											asset, file.filename()
										));
									}
									_ => {}
								}
							}
						}
					}
				}

				PageContent::Image(img) => {
					// Image directement au niveau de la page
					if let Some(bytes) = img.data() {
						let ext = img.extension().unwrap_or("bin");
						let asset = format!("{}/{}_{}.{}", &output, &fname, "img", ext);
						std::fs::write(&asset, bytes)?;
						html.push_str(&format!("<img src=\"{}\" alt=\"image\" />\n", asset));
					}
				}

				PageContent::EmbeddedFile(file) => {
					let asset = format!("{}/{}_{}", &output, &fname, file.filename());
					std::fs::write(&asset, file.data())?;
					html.push_str(&format!(
						"<a href=\"{}\">{}</a>\n", asset, file.filename()
					));
				}

				PageContent::Ink(ink) => {
					// À toi de voir : rendu SVG/PNG, ou info de bounding box
					// Ex : html.push_str("<!-- Ink strokes non rendus -->\n");
					let bbox = ink.bounding_box();
					if let Some(bb) = bbox {
						html.push_str(&format!(
							"<!-- Ink bbox: left={} top={} w={} h={} -->\n",
							bb.y(), bb.x(), bb.width(), bb.height()
						));
					}
				}

				PageContent::Unknown => {
					// Rien à faire ; garder une trace si besoin
					// html.push_str("<!-- Contenu inconnu -->\n");
				}
            }
        }
        html.push_str("</body>");
        fs::write(format!("{}/{}.html", &output, &fname), html)?;
        }
    }
    println!("OK → HTML dans {}", output);
    Ok(())
}

fn sanitize(s: &str) -> String {
    let mut o = s.replace(|c: char| r#"\/*?:"<>|"#.contains(c), "_");
    if o.len() > 120 { o.truncate(120); }
    o
}
fn escape(s: &str) -> String {
    s.replace('&', "&amp;").replace('<', "&lt;").replace('>', "&gt;")
}

