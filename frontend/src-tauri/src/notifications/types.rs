use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// Bilingual notification strings (UI-07, D-01)
// ---------------------------------------------------------------------------

pub struct NotificationStrings {
    pub app_name: &'static str,
    pub app_name_error: &'static str,
    pub recording_started_named: &'static str,
    pub recording_started_generic: &'static str,
    pub recording_stopped: &'static str,
    pub recording_paused: &'static str,
    pub recording_resumed: &'static str,
    pub transcription_complete_named: &'static str,
    pub transcription_complete_generic: &'static str,
    pub meeting_reminder_named: &'static str,
    pub meeting_reminder_generic: &'static str,
    pub test_notification: &'static str,
}

pub const NOTIF_EN: NotificationStrings = NotificationStrings {
    app_name: "Meetily",
    app_name_error: "Meetily Error",
    recording_started_named: "Recording started for meeting: {}",
    recording_started_generic: "Recording has started. Please inform others in the meeting that you are recording.",
    recording_stopped: "Recording has been stopped and saved",
    recording_paused: "Recording has been paused",
    recording_resumed: "Recording has been resumed",
    transcription_complete_named: "Transcription completed and saved to: {}",
    transcription_complete_generic: "Transcription has been completed",
    meeting_reminder_named: "Meeting '{}' starts in {} minutes",
    meeting_reminder_generic: "Meeting starts in {} minutes",
    test_notification: "This is a test notification from Meetily. If you can see this, notifications are working correctly!",
};

pub const NOTIF_AR: NotificationStrings = NotificationStrings {
    app_name: "Meetily",
    app_name_error: "خطأ في Meetily",
    recording_started_named: "بدأ تسجيل الاجتماع: {}",
    recording_started_generic: "بدأ التسجيل. يرجى إبلاغ المشاركين بأن هذا الاجتماع يتم تسجيله.",
    recording_stopped: "تم إيقاف التسجيل وحفظه",
    recording_paused: "تم إيقاف التسجيل مؤقتا",
    recording_resumed: "تم استئناف التسجيل",
    transcription_complete_named: "اكتمل النسخ وتم حفظه في: {}",
    transcription_complete_generic: "اكتمل النسخ",
    meeting_reminder_named: "يبدأ اجتماع '{}' خلال {} دقائق",
    meeting_reminder_generic: "يبدأ الاجتماع خلال {} دقائق",
    test_notification: "هذا إشعار تجريبي من Meetily. إذا كنت ترى هذا، فالإشعارات تعمل بشكل صحيح!",
};

fn notif_strings(locale: &str) -> &'static NotificationStrings {
    match locale {
        "ar" => &NOTIF_AR,
        _ => &NOTIF_EN,
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Notification {
    pub id: Option<String>,
    pub title: String,
    pub body: String,
    pub notification_type: NotificationType,
    pub priority: NotificationPriority,
    pub timeout: NotificationTimeout,
    pub icon: Option<String>,
    pub sound: bool,
    pub actions: Vec<NotificationAction>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NotificationType {
    RecordingStarted,
    RecordingStopped,
    RecordingPaused,
    RecordingResumed,
    TranscriptionComplete,
    MeetingReminder(u64), // Duration in minutes
    SystemError(String),
    Test, // For testing notifications
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NotificationPriority {
    Low,
    Normal,
    High,
    Critical,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NotificationTimeout {
    Never,
    Seconds(u64),
    Default,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationAction {
    pub id: String,
    pub title: String,
    pub action_type: NotificationActionType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NotificationActionType {
    Button,
    Reply,
}

impl Notification {
    pub fn new(title: impl Into<String>, body: impl Into<String>, notification_type: NotificationType) -> Self {
        Self {
            id: None,
            title: title.into(),
            body: body.into(),
            notification_type,
            priority: NotificationPriority::Normal,
            timeout: NotificationTimeout::Default,
            icon: None,
            sound: true,
            actions: vec![],
        }
    }

    pub fn with_priority(mut self, priority: NotificationPriority) -> Self {
        self.priority = priority;
        self
    }

    pub fn with_timeout(mut self, timeout: NotificationTimeout) -> Self {
        self.timeout = timeout;
        self
    }

    pub fn with_sound(mut self, sound: bool) -> Self {
        self.sound = sound;
        self
    }

    pub fn with_icon(mut self, icon: impl Into<String>) -> Self {
        self.icon = Some(icon.into());
        self
    }

    pub fn with_id(mut self, id: impl Into<String>) -> Self {
        self.id = Some(id.into());
        self
    }

    pub fn add_action(mut self, action: NotificationAction) -> Self {
        self.actions.push(action);
        self
    }
}

impl Default for NotificationPriority {
    fn default() -> Self {
        NotificationPriority::Normal
    }
}

impl Default for NotificationTimeout {
    fn default() -> Self {
        NotificationTimeout::Default
    }
}

// Helper functions for creating common notifications (locale-aware via UI-07)
impl Notification {
    pub fn recording_started(meeting_name: Option<String>) -> Self {
        let s = notif_strings(&crate::preferences::read().ui_locale);
        let body = match meeting_name {
            Some(name) => s.recording_started_named.replace("{}", &name),
            None => s.recording_started_generic.to_string(),
        };

        Notification::new(s.app_name, body, NotificationType::RecordingStarted)
            .with_priority(NotificationPriority::High)
            .with_timeout(NotificationTimeout::Seconds(5))
    }

    pub fn recording_stopped() -> Self {
        let s = notif_strings(&crate::preferences::read().ui_locale);
        Notification::new(
            s.app_name,
            s.recording_stopped,
            NotificationType::RecordingStopped,
        )
        .with_priority(NotificationPriority::Normal)
        .with_timeout(NotificationTimeout::Seconds(3))
    }

    pub fn recording_paused() -> Self {
        let s = notif_strings(&crate::preferences::read().ui_locale);
        Notification::new(
            s.app_name,
            s.recording_paused,
            NotificationType::RecordingPaused,
        )
        .with_priority(NotificationPriority::Normal)
        .with_timeout(NotificationTimeout::Seconds(3))
    }

    pub fn recording_resumed() -> Self {
        let s = notif_strings(&crate::preferences::read().ui_locale);
        Notification::new(
            s.app_name,
            s.recording_resumed,
            NotificationType::RecordingResumed,
        )
        .with_priority(NotificationPriority::Normal)
        .with_timeout(NotificationTimeout::Seconds(3))
    }

    pub fn transcription_complete(file_path: Option<String>) -> Self {
        let s = notif_strings(&crate::preferences::read().ui_locale);
        let body = match file_path {
            Some(path) => s.transcription_complete_named.replace("{}", &path),
            None => s.transcription_complete_generic.to_string(),
        };

        Notification::new(s.app_name, body, NotificationType::TranscriptionComplete)
            .with_priority(NotificationPriority::Normal)
            .with_timeout(NotificationTimeout::Seconds(5))
    }

    pub fn meeting_reminder(minutes_until: u64, meeting_title: Option<String>) -> Self {
        let s = notif_strings(&crate::preferences::read().ui_locale);
        let body = match meeting_title {
            Some(title) => s
                .meeting_reminder_named
                .replacen("{}", &title, 1)
                .replacen("{}", &minutes_until.to_string(), 1),
            None => s
                .meeting_reminder_generic
                .replace("{}", &minutes_until.to_string()),
        };

        Notification::new(
            s.app_name,
            body,
            NotificationType::MeetingReminder(minutes_until),
        )
        .with_priority(NotificationPriority::High)
        .with_timeout(NotificationTimeout::Seconds(10))
    }

    pub fn system_error(error: impl Into<String>) -> Self {
        let s = notif_strings(&crate::preferences::read().ui_locale);
        let error_string = error.into();
        Notification::new(
            s.app_name_error,
            error_string.clone(),
            NotificationType::SystemError(error_string),
        )
        .with_priority(NotificationPriority::Critical)
        .with_timeout(NotificationTimeout::Never)
    }

    pub fn test_notification() -> Self {
        let s = notif_strings(&crate::preferences::read().ui_locale);
        Notification::new(s.app_name, s.test_notification, NotificationType::Test)
            .with_priority(NotificationPriority::Normal)
            .with_timeout(NotificationTimeout::Seconds(5))
    }
}