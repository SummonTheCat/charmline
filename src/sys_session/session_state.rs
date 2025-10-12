// ----- Imports ----- //

use std::{
    collections::HashMap,
    sync::{Arc, Mutex, OnceLock},
    time::{Duration, Instant},
};

use uuid::Uuid;

// ----- Global Session Management ----- //

static SESSION_MANAGER: OnceLock<SessionManager> = OnceLock::new();

pub fn init_session_manager() {
    SESSION_MANAGER
        .set(SessionManager::new())
        .expect("SessionManager already initialized");
}

pub fn get_session_manager() -> &'static SessionManager {
    SESSION_MANAGER
        .get()
        .expect("SessionManager not initialized")
}

// ----- Session Manager Structure ----- //

#[derive(Clone, Debug)]
pub struct SessionManager {
    pub sessions: Arc<Mutex<HashMap<String, Session>>>,
}

impl SessionManager {
    pub fn new() -> Self {
        Self {
            sessions: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn create_session(&self, timeout_secs: u64) -> Session {
        let id = Uuid::new_v4().to_string();
        let session = Session {
            session_id: id.clone(),
            session_start: Instant::now(),
            session_timeout: Instant::now() + Duration::from_secs(timeout_secs),
            session_chat: String::new(),
        };
        self.update_session(session.clone());
        session
    }

    pub fn get_session(&self, session_id: &str) -> Option<Session> {
        self.sessions.lock().unwrap().get(session_id).cloned()
    }

    pub fn update_session(&self, session: Session) {
        self.sessions
            .lock()
            .unwrap()
            .insert(session.session_id.clone(), session);
    }

    pub fn tick(&self) {
        let mut map = self.sessions.lock().unwrap();
        let now = Instant::now();
        map.retain(|_, s| s.session_timeout > now);
        for (id, session) in map.iter() {
            println!(
                "Session ID: {}, Expires In: {} secs",
                id,
                (session.session_timeout - now).as_secs()
            );
        }
    }
}

// ----- Session Structure ----- //

#[derive(Clone, Debug)]
pub struct Session {
    pub session_id: String,
    pub session_start: Instant,
    pub session_timeout: Instant,
    pub session_chat: String,
}

impl Session {
    pub fn time_remaining(&self) -> u64 {
        let now = Instant::now();
        if self.session_timeout > now {
            (self.session_timeout - now).as_secs()
        } else {
            0
        }
    }
}

// ----- Session Artifact Structures ----- //

// A JSON Session Artifact at the end of a session
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct SessionArtifact {
    #[serde(rename = "sessionId")]
    pub session_id: String,
    #[serde(rename = "sessionTranscript")]
    pub session_transcript: String,
    #[serde(rename = "sessionStart")]
    pub session_start: String,
    #[serde(rename = "sessionEnd")]
    pub session_end: String,
    #[serde(rename = "summary")]
    pub summary: SessionSummary,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct SessionSummary {
    #[serde(rename = "callerName")]
    pub caller_name: String,
    #[serde(rename = "callerNumber")]
    pub caller_number: String,
    #[serde(rename = "company")]
    pub company: String,
    #[serde(rename = "solutionType")]
    pub solution_type: String,
    #[serde(rename = "projectDetails")]
    pub project_details: String,
    #[serde(rename = "additionalNotes")]
    pub additional_notes: String,
    #[serde(rename = "tags")]
    pub tags: Vec<String>, // New field for relevant tags
}
