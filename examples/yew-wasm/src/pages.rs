#![allow(unused)]

use crate::error::ApiError;
use crate::types::{to_jsvalue, PDFFindControllerOptions, PDFViewerOptions};
use crate::view;
use crate::types::*;
use serde::Serialize;
use serde_json::json;
use std::collections::HashMap;
use std::rc::Rc;
use stylist::{css, yew::styled_component};
use wasm_bindgen::{JsCast, JsValue};
use yew::prelude::*;
use yew_hooks::{use_async, use_async_with_options, UseAsyncOptions};

#[derive(Debug, Clone, PartialEq)]
pub struct PdfDocument {
    pub num_pages: usize,
}

#[derive(Properties, PartialEq, Debug)]
pub struct PdfPagesProps {
    pub scale: f64,
    pub url: String,
    pub set_pdf: Callback<PdfDocument>,
    pub set_progress: Callback<f64>,
    pub set_current_page: Callback<usize>,
    pub set_viewer: Callback<PDFViewer>,
    pub set_find_controller: Callback<PDFFindController>,
}

#[function_component]
pub fn PdfPages(
    PdfPagesProps {
        scale,
        url,
        set_pdf,
        set_progress,
        set_current_page,
        set_viewer,
        set_find_controller,
    }: &PdfPagesProps,
) -> Html {
    let pdf_viewer: UseStateHandle<Option<PDFViewer>> = use_state(|| None);
    let pdf_viewer: UseStateHandle<Option<PDFViewer>> = use_state(|| None);

    let set_up_viewer = || {
        // PDF Link Service
        // LinkServiceArgs
        let event_bus: EventBus = EventBus::new();
        let opts = PDFLinkServiceOptions {
            event_bus: None,
            external_link_target: None,
            external_link_rel: None,
            ignore_destination_zoom: None,
        };
        let opts = serde_wasm_bindgen::to_value(&opts).unwrap();
        let linker = PDFLinkService::new(&opts);
        //
        let linker_clone = {
            let c = linker.clone();
            c.dyn_into::<PDFLinkService>()
        }
        .unwrap();

        // PDF Viewer
        // ViewerArgs
        let viewer_args = PDFViewerOptions {
            event_bus,
            link_service: Some(linker_clone),
            download_manager: None,
            find_controller: None,
            scripting_manager: None,
            rendering_queue: None,
            remove_page_borders: None,
            text_layer_mode: None,
            annotation_mode: None,
            annotation_editor_mode: None,
            annotation_editor_highlight_colors: None,
            image_resources_path: None,
            enable_print_auto_rotate: None,
            max_canvas_pixels: None,
            l10n: None,
            enable_permissions: None,
            page_colors: None,
            container: todo!(),
            viewer: todo!(),
        };
        let opts = to_jsvalue(viewer_args).unwrap();
        let viewer = PDFViewer::new(&opts);
        linker.set_viewer(&viewer);

        // PDF Find Controller
        let args = PDFFindControllerOptions {
            link_service: linker,
            event_bus,
            update_matches_count_on_progress: None,
        };
        let opts = to_jsvalue(args).unwrap();
        let find_controller = PDFFindController::new(&opts);
        viewer.set_find_controller(&find_controller);

        // Set external ref
        set_viewer.emit(viewer);
        set_find_controller.emit(find_controller);
    };

    let onscroll = { Callback::from(move |e| {}) };

    // This fetch the pdf docs from server
    let stream = use_async({ async { Ok::<_, ApiError>(()) } });

    html! {
        <div id="viewer-container" {onscroll} >
        </div>
    }
}

#[styled_component]
pub fn PdfViewerComponent() -> Html {
    let class_name = css!(
        r#"
		.textLayer {
		position: absolute;
		left: 0;
		top: 0;
		right: 0;
		bottom: 0;
		overflow: hidden;
		opacity: 0.2;
		line-height: 1.0;
		}

		.textLayer > div {
		color: transparent;
		position: absolute;
		white-space: pre;
		cursor: text;
		-webkit-transform-origin: 0% 0%;
		-moz-transform-origin: 0% 0%;
		-o-transform-origin: 0% 0%;
		-ms-transform-origin: 0% 0%;
		transform-origin: 0% 0%;
		}

		.textLayer .highlight {
		margin: -1px;
		padding: 1px;
		background-color: #2078A9;
		}


		.textLayer .highlight.selected {
			background-color: #0094FF;
		}
		
		.textLayer .highlight.begin {
			border-radius: 4px 0px 0px 4px;
		}
		
		.textLayer .highlight.end {
			border-radius: 0px 4px 4px 0px;
		}
		
		.textLayer .highlight.middle {
			border-radius: 0px;
		}


		.textLayer ::selection { background: rgb(0,0,255); }
		.textLayer ::-moz-selection { background: rgb(0,0,255); }


		.textLayer .endOfContent {
			display: block;
			position: absolute;
			left: 0px;
			top: 100%;
			right: 0px;
			bottom: 0px;
			z-index: -1;
			cursor: default;
			-webkit-user-select: none;
			-ms-user-select: none;
			-moz-user-select: none;
		}


		.textLayer .endOfContent.active {
			top: 0px;
		}
		

		.pdfViewer {
			height: 100%;
			padding-top: 22px;
			margin-left: 190px;
			transition: all 0.5s;
			background-color: #F1F3F7;
		
		}

		.pdfViewer .canvasWrapper {
        	overflow: hidden;
      	}
      
      	.pdfViewer .page {
			direction: ltr;
			width: 816px;
			height: 1056px;
			margin: 5px auto 5px auto;
			position: relative;
			overflow: visible;
			background-clip: content-box;
			background-color: white;
		}
      
		.pdfViewer.removePageBorders .page {
			margin: 0px auto 10px auto;
			border: none;
		}
		
		.pdfViewer.singlePageView {
			display: inline-block;
		}
		
		.pdfViewer.singlePageView .page {
			margin: 0;
			border: none;
		}
		
		.pdfViewer .page canvas {
			margin: 0;
			display: block;
		}
    "#
    );

    html! {
        <>
        <div id="pdf-pages" class={class_name}>
        </div>
        </>
    }
}
