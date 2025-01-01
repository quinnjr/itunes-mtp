mod app_state;
mod errors;
mod mtp;

use iced::{
    widget::{button, column, container, pick_list, text},
    executor, Application, Command, Element, Length, Settings, Theme,
};

use std::{rc::Rc, cell::RefCell};
use crate::app_state::AppState;

#[derive(Debug, Clone)]
pub enum Message {
    SelectLibrary,
    DeviceSelected(String),
    StartSync,
    StatusUpdate(String),
}

pub struct ItunesMtpSync {
    state: Rc<RefCell<AppState>>,
    devices: Vec<String>,
    selected_device: Option<String>,
    status: String,
    can_sync: bool,
}

impl iced::Application for ItunesMtpSync {
    type Message = Message;
    type Theme = iced::Theme;
    type Executor = iced::executor::Default;
    type Flags = ();

    fn new(_flags: ()) -> (Self, iced::Command<Message>) {
        (
            Self {
                state: Rc::new(RefCell::new(AppState::default())),
                devices: Vec::new(),
                selected_device: None,
                status: String::from("Ready"),
                can_sync: false,
            },
            iced::Command::none(),
        )
    }

    fn title(&self) -> String {
        String::from("iTunes MTP Sync")
    }

    fn update(&mut self, message: Message) -> Command<Message> {
        match message {
            Message::SelectLibrary => {
                // TODO: Implement file picker
                Command::none()
            }
            Message::DeviceSelected(device) => {
                self.selected_device = Some(device);
                self.can_sync = true;
                Command::none()
            }
            Message::StartSync => {
                // TODO: Implement sync
                Command::none()
            }
            Message::StatusUpdate(status) => {
                self.status = status;
                Command::none()
            }
        }
    }

    fn view(&self) -> Element<Message> {
        let select_library = button("Select Library...")
            .on_press(Message::SelectLibrary);

        let device_picker = pick_list(
            &self.devices,
            self.selected_device.clone(),
            Message::DeviceSelected,
        );

        let status = text(&self.status);

        let sync_button = if self.can_sync {
            button("Sync").on_press(Message::StartSync)
        } else {
            button("Sync")
        };

        container(
            column![
                select_library,
                device_picker,
                status,
                sync_button,
            ]
            .spacing(10)
            .padding(20),
        )
        .width(Length::Fill)
        .height(Length::Fill)
        .center_x()
        .center_y()
        .into()
    }
}

fn main() -> iced::Result {
    ItunesMtpSync::run(Settings::default())
}
