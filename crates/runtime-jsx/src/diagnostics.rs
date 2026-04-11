use std::collections::VecDeque;

pub const LOG_HISTORY_LIMIT: usize = 200;
pub const LOG_MESSAGE_LIMIT_BYTES: usize = 4 * 1024;

const LOG_TRUNCATION_SUFFIX: &str = " [truncated]";

pub type RuntimeLogBuffer = VecDeque<String>;

pub fn push_log(logs: &mut RuntimeLogBuffer, message: impl Into<String>) {
    logs.push_back(truncate_log_message(message.into()));
    trim_logs(logs);
}

pub fn extend_logs(logs: &mut RuntimeLogBuffer, messages: impl IntoIterator<Item = String>) {
    for message in messages {
        push_log(logs, message);
    }
}

fn trim_logs(logs: &mut RuntimeLogBuffer) {
    while logs.len() > LOG_HISTORY_LIMIT {
        logs.pop_front();
    }
}

fn truncate_log_message(mut message: String) -> String {
    if message.len() <= LOG_MESSAGE_LIMIT_BYTES {
        return message;
    }

    let target_len = LOG_MESSAGE_LIMIT_BYTES.saturating_sub(LOG_TRUNCATION_SUFFIX.len());
    let mut cutoff = target_len;
    while !message.is_char_boundary(cutoff) {
        cutoff -= 1;
    }
    message.truncate(cutoff);
    message.push_str(LOG_TRUNCATION_SUFFIX);
    message
}

#[cfg(test)]
mod tests {
    use super::{
        push_log, RuntimeLogBuffer, LOG_HISTORY_LIMIT, LOG_MESSAGE_LIMIT_BYTES,
        LOG_TRUNCATION_SUFFIX,
    };

    #[test]
    fn keeps_only_the_latest_log_entries() {
        let mut logs = RuntimeLogBuffer::new();

        for index in 0..(LOG_HISTORY_LIMIT + 5) {
            push_log(&mut logs, format!("log {index}"));
        }

        assert_eq!(logs.len(), LOG_HISTORY_LIMIT);
        assert_eq!(logs.front().map(String::as_str), Some("log 5"));
        assert_eq!(logs.back().map(String::as_str), Some("log 204"));
    }

    #[test]
    fn truncates_large_log_messages_on_utf8_boundaries() {
        let mut logs = RuntimeLogBuffer::new();
        let message = "🙂".repeat(LOG_MESSAGE_LIMIT_BYTES);

        push_log(&mut logs, message);

        let stored = logs.back().expect("log should be stored");
        assert!(stored.len() <= LOG_MESSAGE_LIMIT_BYTES);
        assert!(stored.ends_with(LOG_TRUNCATION_SUFFIX));
        assert!(stored.is_char_boundary(stored.len()));
    }
}
