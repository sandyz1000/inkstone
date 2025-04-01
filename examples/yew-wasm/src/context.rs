use std::rc::Rc;

use yew::{Reducible, UseReducerHandle};


pub type PDFViewerContext = UseReducerHandle<PDFViewer>;
pub type PDFFindControllerContext = UseReducerHandle<PDFFindController>;
pub type PDFLinkServiceContext = UseReducerHandle<PDFLinkService>;


impl Reducible for PDFViewer {
    type Action = PDFViewer;

    fn reduce(self: Rc<Self>, action: Self::Action) -> Rc<Self> {
        todo!()
    }
}

impl Reducible for PDFLinkService {
    type Action = PDFLinkService;

    fn reduce(self: Rc<Self>, action: Self::Action) -> Rc<Self> {
        todo!()
    }
}

