use yew::prelude::*;


#[derive(Clone, PartialEq, Properties)]
pub struct ProgressBarProps {
    pub progress: f64,
}

#[function_component]
pub fn ProgressBar(props: &ProgressBarProps) -> Html {
    html! {
        <>
        if props.progress > 0.0 && props.progress < 100.0 {
            <div class="progress-bar">
                <div class="progress">
                    <div class="progress-percent" style={format!("width: {} %", props.progress)}></div>
                </div>
            </div>
        }    
        </>
    }
}
