use std::collections::HashMap;

use serde::Serialize;
use wasm_bindgen::{JsCast, JsValue};


pub struct PDFFindController;

pub struct  PDFLinkService;

pub struct PDFViewer;



#[derive(Debug, Serialize, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ScrollMatchViewArgs {
    element: Option<HtmlDivElementL>, 
    selected_left: u32, 
    page_index: u32,
    match_index: u32
}


#[derive(Debug, Serialize, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct PDFLinkServiceOptions {
    pub event_bus: Option<EventBus>,
    /**
     * - Specifies the `target` attribute
     * for external links. Must use one of the values from {LinkTarget}.
     * Defaults to using no target.
     */
    pub external_link_target: Option<f32>,
    /**
     * - Specifies the `rel` attribute for
     * external links. Defaults to stripping the referrer.
     */
    pub external_link_rel: Option<String>,
    /**
     * - Ignores the zoom argument,
     * thus preserving the current zoom level in the viewer, when navigating
     * to internal destinations. The default value is `false`.
     */
    pub ignore_destination_zoom: Option<bool>,
}

#[derive(Serialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct PDFViewerOptions {
    /// The container for the viewer element.
    pub container: HtmlDivElementL,
    /// The viewer element.
    pub viewer: Option<HtmlDivElementL>,
    /// The application event bus.
    pub event_bus: EventBus,
    /// The navigation/linking service.
    pub link_service: Option<PDFLinkService>,
    /// The download manager component.
    pub download_manager: Option<DownloadManager>,
    /// The find controller component.
    pub find_controller: Option<PDFFindController>,
    /// The scripting manager component.
    pub scripting_manager: Option<PDFScriptingManager>,
    /// The rendering queue object.
    pub rendering_queue: Option<PDFRenderingQueue>,
    /// Removes the border shadow around the pages. The default value is `false`.
    pub remove_page_borders: Option<bool>,
    /// Controls if the text layer used for selection and searching is created. The constants from `TextLayerMode`
    /// should be used. The default value is `TextLayerMode::ENABLE`.
    pub text_layer_mode: Option<i32>,
    /// Controls if the annotation layer is created, and if interactive form elements or `AnnotationStorage`-data
    /// are being rendered. The constants from `AnnotationMode` should be used. The default value is `AnnotationMode::ENABLE_FORMS`.
    pub annotation_mode: Option<i32>,
    /// Enables the creation and editing of new Annotations. The constants from `AnnotationEditorType` should be used.
    /// The default value is `AnnotationEditorType::NONE`.
    pub annotation_editor_mode: Option<i32>,
    /// A comma separated list of colors to propose to highlight some text in the pdf.
    pub annotation_editor_highlight_colors: Option<String>,
    /// Path for image resources, mainly for annotation icons. Include trailing slash.
    pub image_resources_path: Option<String>,
    /// Enables automatic rotation of landscape pages upon printing. The default is `false`.
    pub enable_print_auto_rotate: Option<bool>,
    /// The maximum supported canvas size in total pixels, i.e. width * height. Use `-1` for no limit, or `0` for CSS-only zooming.
    /// The default value is 4096 * 8192 (32 mega-pixels).
    pub max_canvas_pixels: Option<i32>,
    /// Localization service.
    pub l10n: Option<GenericL10n>,
    /// Enables PDF document permissions, when they exist. The default value is `false`.
    pub enable_permissions: Option<bool>,
    /// Overwrites background and foreground colors with user defined ones in order to improve readability in high contrast mode.
    pub page_colors: Option<HashMap<String, String>>,
}

#[derive(Serialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct PDFFindControllerOptions {
    pub link_service: PDFLinkService,
    pub event_bus: EventBus,
    pub update_matches_count_on_progress: Option<bool>,
}

pub fn to_jsvalue<T>(rust_obj: T) -> Result<JsValue, JsValue>
where
    T: serde::Serialize + Clone + std::fmt::Debug + PartialEq + ?Sized,
{
    serde_wasm_bindgen::to_value(&rust_obj).map_err(|e| JsValue::from_str(&e.to_string()))
}

pub fn to_rsobj<T>(val: JsValue) -> Result<T, JsValue>
where
    T: serde::de::DeserializeOwned + Clone + std::fmt::Debug + PartialEq,
{
    let res: T = serde_wasm_bindgen::from_value(val)?;
    Ok(res)
}
