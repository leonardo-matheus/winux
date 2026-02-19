// Alarm notifications

use crate::data::alarm::Alarm;
use std::process::Command;

/// Trigger an alarm notification
pub fn trigger_alarm(alarm: &Alarm) {
    let title = if alarm.label.is_empty() {
        "Alarme".to_string()
    } else {
        alarm.label.clone()
    };

    let body = format!("{:02}:{:02}", alarm.hour, alarm.minute);

    send_notification(&title, &body, "alarm-symbolic", true);
    play_alarm_sound(&alarm.sound);
}

/// Trigger timer finished notification
pub fn trigger_timer_finished() {
    send_notification(
        "Timer Finalizado",
        "O tempo acabou!",
        "alarm-symbolic",
        true,
    );
    play_alarm_sound("default");
}

/// Send a desktop notification
fn send_notification(title: &str, body: &str, icon: &str, urgent: bool) {
    let mut args = vec![
        "-a", "Winux Clock",
        "-i", icon,
        title,
        body,
    ];

    if urgent {
        args.insert(0, "-u");
        args.insert(1, "critical");
    }

    if let Err(e) = Command::new("notify-send").args(&args).spawn() {
        tracing::warn!("Failed to send notification: {}", e);
    }
}

/// Play alarm sound using paplay or similar
fn play_alarm_sound(sound_name: &str) {
    let sound_file = match sound_name {
        "gentle" => "/usr/share/sounds/freedesktop/stereo/alarm-clock-elapsed.oga",
        "digital" => "/usr/share/sounds/freedesktop/stereo/complete.oga",
        "classic" => "/usr/share/sounds/freedesktop/stereo/bell.oga",
        "nature" => "/usr/share/sounds/freedesktop/stereo/message.oga",
        _ => "/usr/share/sounds/freedesktop/stereo/alarm-clock-elapsed.oga",
    };

    // Try to play sound with paplay (PulseAudio)
    if let Err(_) = Command::new("paplay").arg(sound_file).spawn() {
        // Fallback to aplay (ALSA)
        if let Err(_) = Command::new("aplay").arg(sound_file).spawn() {
            // Try pw-play (PipeWire)
            if let Err(e) = Command::new("pw-play").arg(sound_file).spawn() {
                tracing::warn!("Failed to play alarm sound: {}", e);
            }
        }
    }
}

/// Show snooze dialog
pub fn show_snooze_dialog(alarm: &Alarm) -> bool {
    // For now, we'll just return true to indicate snooze was pressed
    // In a full implementation, this would show a proper dialog
    tracing::info!("Snooze requested for alarm: {}", alarm.label);
    true
}

/// Dismiss alarm
pub fn dismiss_alarm(alarm: &Alarm) {
    tracing::info!("Alarm dismissed: {}", alarm.label);
    // Stop the sound if it's playing
    stop_alarm_sound();
}

/// Stop any playing alarm sound
pub fn stop_alarm_sound() {
    // In a real implementation, we would track the sound process and kill it
    // For now, we'll use a simple approach
    let _ = Command::new("pkill").args(["-f", "paplay"]).spawn();
    let _ = Command::new("pkill").args(["-f", "pw-play"]).spawn();
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::alarm::RepeatDays;

    #[test]
    fn test_alarm_creation() {
        let alarm = Alarm {
            id: 1,
            hour: 7,
            minute: 30,
            label: "Wake up".to_string(),
            enabled: true,
            repeat: RepeatDays::weekdays(),
            snooze_minutes: 10,
            sound: "default".to_string(),
        };

        assert_eq!(alarm.time_string(), "07:30");
    }
}
