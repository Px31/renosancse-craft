use iced::alignment::Vertical;
use iced::widget::{button, column, container, row, scrollable, text};
use iced::{Element, Length};

use crate::app::RenosancseApp;
use crate::model::{AppMessage, ItemStatus};

/// Build the whole window. Kept as free functions so the layout stays
/// testable and the state type stays out of the widget code.
pub fn render(app: &RenosancseApp) -> Element<'_, AppMessage> {
    let ffmpeg_status = if app.ffmpeg_available {
        "FFmpeg: found"
    } else {
        "FFmpeg: NOT FOUND on PATH"
    };

    let output_dir = app
        .config
        .output_dir
        .as_ref()
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_else(|| "same as input".into());

    let mut queue_children: Vec<Element<AppMessage>> = Vec::new();
    for (index, item) in app.items.iter().enumerate() {
        let detected = item.format.map(|f| f.label()).unwrap_or("unknown");
        let probed = item
            .probe_info
            .as_ref()
            .map(|p| p.summary())
            .map(|s| format!(" · probe: {s}"))
            .unwrap_or_default();
        let name = item
            .path
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| "?".into());
        let rec = item
            .recommendation
            .as_ref()
            .map(|r| r.reason.clone())
            .unwrap_or_else(|| "Analyze to get a recommendation".into());

        let detail = if probed.is_empty() {
            format!("[{detected}] {}", item.detection_note)
        } else {
            format!("[{detected}] {}{probed}", item.detection_note)
        };

        let row_children = vec![
            text(name).width(Length::FillPortion(3)).into(),
            text(detail).width(Length::FillPortion(3)).into(),
            text(rec).width(Length::FillPortion(4)).into(),
            text(status_label(&item.status))
                .width(Length::FillPortion(3))
                .into(),
            button("remove")
                .on_press(AppMessage::RemoveItem(index))
                .into(),
        ];
        queue_children.push(
            row(row_children)
                .spacing(12)
                .align_y(Vertical::Center)
                .into(),
        );
    }

    let title_children = vec![
        text("Renosancse Craft").size(28).into(),
        text(ffmpeg_status).into(),
    ];
    let toolbar_children = vec![
        button("Add files…")
            .on_press(AppMessage::BrowseFiles)
            .into(),
        button("Analyze").on_press(AppMessage::AnalyzeAll).into(),
        button("Convert all")
            .on_press(AppMessage::ConvertAll)
            .into(),
        button("Clear finished")
            .on_press(AppMessage::ClearFinished)
            .into(),
    ];

    let main_children = vec![
        row(title_children)
            .spacing(12)
            .align_y(Vertical::Center)
            .into(),
        row(toolbar_children).spacing(8).into(),
        text(format!("Output directory: {output_dir}"))
            .size(14)
            .into(),
        scrollable(column(queue_children).spacing(6))
            .height(Length::Fill)
            .into(),
        text(app.status.clone()).size(14).into(),
    ];

    container(
        column(main_children)
            .spacing(12)
            .padding(16)
            .width(Length::Fill)
            .height(Length::Fill),
    )
    .width(Length::Fill)
    .height(Length::Fill)
    .into()
}

fn status_label(status: &ItemStatus) -> String {
    match status {
        ItemStatus::Queued => "queued".to_string(),
        ItemStatus::Converting => "converting…".to_string(),
        ItemStatus::Done(path) => format!("done → {}", path.to_string_lossy()),
        ItemStatus::Failed(err) => format!("error: {err}"),
    }
}
