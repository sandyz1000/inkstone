use glyphmatcher::FontDb;
use std::borrow::Cow;
use pdf::error::{ PdfError, Result };
use pdf::font::Font as PdfFont;
use pdf::object::*;
use std::collections::HashMap;
use std::ops::Deref;
use std::path::PathBuf;

use super::FontEntry;
use inkfont;
use globalcache::{ sync::SyncCache, ValueSize };
use std::hash::{ Hash, Hasher };
use std::sync::Arc;

#[derive(Clone)]
pub struct FontRc(Arc<dyn inkfont::Font + Send + Sync + 'static>);
impl ValueSize for FontRc {
    #[inline]
    fn size(&self) -> usize {
        1 // TODO
    }
}
impl From<Box<dyn inkfont::Font + Send + Sync + 'static>> for FontRc {
    #[inline]
    fn from(f: Box<dyn inkfont::Font + Send + Sync + 'static>) -> Self {
        FontRc(f.into())
    }
}
impl Deref for FontRc {
    type Target = dyn inkfont::Font + Send + Sync + 'static;
    #[inline]
    fn deref(&self) -> &Self::Target {
        &*self.0
    }
}
impl PartialEq for FontRc {
    #[inline]
    fn eq(&self, rhs: &Self) -> bool {
        Arc::as_ptr(&self.0) == Arc::as_ptr(&rhs.0)
    }
}
impl Eq for FontRc {}
impl Hash for FontRc {
    #[inline]
    fn hash<H: Hasher>(&self, state: &mut H) {
        Arc::as_ptr(&self.0).hash(state)
    }
}
pub struct StandardCache {
    inner: Arc<SyncCache<String, Option<FontRc>>>,
    dir: PathBuf,
    fonts: HashMap<String, String>,
    dump: Dump,
    font_db: Option<FontDb>,
    require_unique_unicode: bool,
}
impl StandardCache {
    pub fn new() -> Self {
        let standard_fonts = PathBuf::from(
            std::env
                ::var_os("STANDARD_FONTS")
                .expect(
                    "STANDARD_FONTS is not set. Please check https://github.com/pdf-rs/pdf_render/#fonts for instructions."
                )
        );
        let data = standard_fonts.read_file("fonts.json").expect("can't read fonts.json");
        let fonts: HashMap<String, String> = serde_json
            ::from_slice(&data)
            .expect("fonts.json is invalid");

        let dump = match std::env::var("DUMP_FONT").as_deref() {
            Err(_) => Dump::Never,
            Ok("always") => Dump::Always,
            Ok("error") => Dump::OnError,
            Ok(_) => Dump::Never,
        };
        let db_path = standard_fonts.join("db");
        let font_db = db_path.is_dir().then(|| FontDb::new(db_path));

        StandardCache {
            inner: SyncCache::new(),
            dir: standard_fonts,
            fonts,
            dump,
            font_db,
            require_unique_unicode: false,
        }
    }

    /// Create an empty cache for environments without standard fonts (e.g., WASM)
    /// This will only work with PDFs that have embedded fonts
    pub fn empty() -> Self {
        StandardCache {
            inner: SyncCache::new(),
            dir: PathBuf::new(),
            fonts: HashMap::new(),
            dump: Dump::Never,
            font_db: None,
            require_unique_unicode: false,
        }
    }

    pub fn require_unique_unicode(&mut self, r: bool) {
        self.require_unique_unicode = r;
    }
}

#[derive(Debug)]
enum Dump {
    Never,
    OnError,
    Always,
}

pub fn load_font(
    font_ref: &MaybeRef<PdfFont>,
    resolve: &impl Resolve,
    cache: &StandardCache
) -> Result<Option<FontEntry>> {
    let pdf_font = font_ref.clone();
    debug!("loading {:?}", pdf_font);

    let font: FontRc = match pdf_font.embedded_data(resolve) {
        Some(Ok(data)) => {
            debug!("loading embedded font");
            let font = inkfont::parse(&data).map_err(|e| PdfError::Other {
                msg: format!("Font Error: {:?}", e),
            });
            if
                matches!(cache.dump, Dump::Always) ||
                (matches!(cache.dump, Dump::OnError) && font.is_err())
            {
                let name = format!(
                    "font_{}",
                    pdf_font.name
                        .as_ref()
                        .map(|s| s.as_str())
                        .unwrap_or("unnamed")
                );
                std::fs::write(&name, &data).unwrap();
                println!("font dumped in {}", name);
            }
            FontRc::from(font?)
        }
        Some(Err(e)) => {
            return Err(e);
        }
        None => {
            debug!("no embedded font.");
            let name = match pdf_font.name {
                Some(ref name) => name.as_str(),
                None => {
                    return Ok(None);
                }
            };
            debug!("loading {name} instead");
            match cache.fonts.get(name).or_else(|| cache.fonts.get("Arial")) {
                Some(file_name) => {
                    let val = cache.inner.get(file_name.clone(), |_| {
                        let data = match cache.dir.read_file(file_name) {
                            Ok(data) => data,
                            Err(e) => {
                                warn!("can't open {} for {:?} {:?}", file_name, pdf_font.name, e);
                                return None;
                            }
                        };
                        match inkfont::parse(&data) {
                            Ok(f) => Some(f.into()),
                            Err(e) => {
                                warn!("Font Error: {:?}", e);
                                return None;
                            }
                        }
                    });
                    match val {
                        Some(f) => f,
                        None => {
                            return Ok(None);
                        }
                    }
                }
                None => {
                    warn!("no font for {:?}", pdf_font.name);
                    return Ok(None);
                }
            }
        }
    };

    Ok(Some(FontEntry::build(font, pdf_font, None, resolve, cache.require_unique_unicode)?))
}

pub trait DirRead: Sized {
    fn read_file(&self, name: &str) -> Result<Cow<'static, [u8]>>;
    fn sub_dir(&self, name: &str) -> Option<Self>;
}

impl DirRead for PathBuf {
    fn read_file(&self, name: &str) -> Result<Cow<'static, [u8]>> {
        std::fs
            ::read(self.join(name))
            .map_err(|e| e.into())
            .map(|d| d.into())
    }
    fn sub_dir(&self, name: &str) -> Option<Self> {
        let sub = self.join(name);
        if sub.is_dir() {
            Some(sub)
        } else {
            None
        }
    }
}

#[cfg(feature = "embed")]
#[derive(rust_embed::Embed)]
#[folder = "$STANDARD_FONTS"]
pub struct EmbeddedStandardFonts;

#[cfg(feature = "embed")]
impl DirRead for EmbeddedStandardFonts {
    fn read_file(&self, name: &str) -> Result<Cow<'static, [u8]>> {
        EmbeddedStandardFonts::get(name)
            .map(|f| f.data)
            .ok_or_else(|| PdfError::Other {
                msg: "Filed {name:?} not embedded".into(),
            })
    }
    fn sub_dir(&self, name: &str) -> Option<Self> {
        None
    }
}
