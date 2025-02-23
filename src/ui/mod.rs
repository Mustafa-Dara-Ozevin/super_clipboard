use std::time::{Duration, Instant};

use arboard::Clipboard;
use iced::widget::scrollable::Properties;
use iced::widget::{svg, Row};
use iced::{
    widget::{container, scrollable, Column},
    Element, Length, Theme,
};
use iced::{Application, Color, Command, Padding, Subscription};
use log::{info, trace};

use crate::daemon;
use crate::gui::{home, settings};
use crate::settings::ThemeType;
use crate::settings::{AppSettings, ClipboardItem};
use crate::update::handle_update;

use self::mouse_listener::mouse_listener;

pub mod item;
pub mod mouse_listener;
pub mod styles;

pub struct MainApp {
    pub settings: AppSettings,
    pub clipboard_ctx: Clipboard,
    pub view: RouterView,
    pub visible: bool,
    pub follow: bool,
    back_icon: svg::Handle,
    tip_icon: svg::Handle,
    dark_icon: svg::Handle,
    light_icon: svg::Handle,
    trash_icon: svg::Handle,
    settings_icon: svg::Handle,
}

#[derive(Debug, Clone)]
pub enum RouterView {
    Home,
    Settings,
}

#[derive(Debug, Clone)]
pub enum MainMessage {
    ClearClipboard,
    ThemeChangedToggle,
    Open(String),
    ChangeSettings(SettingsModified),
    ChangeView(RouterView),
    CheckSettings(Instant),
    DaemonEvent(daemon::Event),
    RemoveClipboard(usize),
    SetClipboard(ClipboardItem),
}

#[derive(Debug, Clone)]
pub enum SettingsModified {
    MaxCapacity(String),
    TickToSave(String),
    StoreClipboard(bool),
    ChangeTransparency(bool),
    DateFormat(String),
    ChangeShortcut(String),
}

impl Application for MainApp {
    type Message = MainMessage;
    type Executor = iced::executor::Default;
    type Theme = Theme;
    type Flags = AppSettings;

    fn new(settings: Self::Flags) -> (Self, Command<Self::Message>) {
        trace!("Creating Iced Application");
        (
            Self {
                settings,
                visible: true,
                follow: false,
                view: RouterView::Home,
                clipboard_ctx: Clipboard::new().unwrap(),
                back_icon: svg::Handle::from_memory(
                    include_bytes!("../../assets/back.svg").to_vec(),
                ),
                tip_icon: svg::Handle::from_memory(
                    include_bytes!("../../assets/hint.svg").to_vec(),
                ),
                dark_icon: svg::Handle::from_memory(
                    include_bytes!("../../assets/night-mode.svg").to_vec(),
                ),
                light_icon: svg::Handle::from_memory(
                    include_bytes!("../../assets/light-mode.svg").to_vec(),
                ),
                trash_icon: svg::Handle::from_memory(
                    include_bytes!("../../assets/trash.svg").to_vec(),
                ),
                settings_icon: svg::Handle::from_memory(
                    include_bytes!("../../assets/settings.svg").to_vec(),
                ),
            },
            Command::none(),
        )
    }

    fn title(&self) -> String {
        String::from("Super Clipboard")
    }

    fn subscription(&self) -> iced::Subscription<Self::Message> {
        trace!("Subscription Batch");
        Subscription::batch(vec![
            daemon::start_daemon(self.settings.shortcut()).map(MainMessage::DaemonEvent),
            iced::time::every(Duration::from_millis(self.settings.tick_save()))
                .map(MainMessage::CheckSettings),
        ])
    }

    fn update(&mut self, message: Self::Message) -> Command<Self::Message> {
        info!("Update Commands");
        handle_update(self, message)
    }

    fn view(&self) -> Element<MainMessage> {
        trace!("Render Application");
        let content: Element<MainMessage> = match self.view {
            RouterView::Home => Column::new()
                .push(home::top_bar(
                    self.settings.get_theme(),
                    self.dark_icon.clone(),
                    self.light_icon.clone(),
                    self.trash_icon.clone(),
                    self.settings_icon.clone(),
                ))
                .push(home::tip_section(self.tip_icon.clone()))
                .push(
                    scrollable(home::show_items(
                        self.settings.format_date(),
                        self.settings.clipboard(),
                    ))
                    .vertical_scroll(Properties::default().width(5.).scroller_width(5.))
                    .height(Length::Fill),
                )
                .spacing(10)
                .padding(10)
                .width(Length::Fill)
                .height(Length::Fill)
                .into(),
            RouterView::Settings => Column::new()
                .push(settings::back_bar(self.back_icon.clone()))
                .push(settings::tip_section(self.tip_icon.clone()))
                .push(
                    Row::new()
                        .push(settings::list_options())
                        .push(settings::list_elements(&self.settings))
                        .spacing(10)
                        .padding(10),
                )
                .padding(Padding::from([10, 0]))
                .width(Length::Fill)
                .height(Length::Fill)
                .into(),
        };

        container(
            mouse_listener(content).on_mouse_exit(MainMessage::DaemonEvent(
                daemon::Event::Message(daemon::Message::ToggleVisibility(false)),
            )),
        )
        .width(Length::Fill)
        .height(Length::Fill)
        .center_x()
        .center_y()
        .into()
    }

    fn theme(&self) -> Theme {
        trace!("Set Theme");
        let primary = Color::from_rgba8(0x77, 0x8f, 0x9b, 1.);
        match self.settings.get_theme() {
            ThemeType::Light => {
                let palette = Theme::Light.palette();
                Theme::custom(iced::theme::Palette {
                    primary,
                    background: Color::from_rgba8(200, 200, 200, 0.025),
                    ..palette
                })
            }
            ThemeType::Dark => {
                let palette = Theme::Dark.palette();
                let bg = palette.background;
                Theme::custom(iced::theme::Palette {
                    primary,
                    background: Color::from_rgba(bg.r, bg.g, bg.b, 0.3),
                    ..palette
                })
            }
        }
    }
}
