use candid::{CandidType, Deserialize};
use ic_cdk::export::candid::Principal;
use ic_cdk_macros::*;
use serde::Serialize;
use std::collections::HashMap;
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};

// Define a global mutex for thread safety
lazy_static::lazy_static! {
    static ref USERS_STORAGE: Mutex<HashMap<String, User>> = Mutex::new(HashMap::new());
    static ref PLOTS_STORAGE: Mutex<HashMap<String, Plot>> = Mutex::new(HashMap::new());
    static ref ACTIVITIES_STORAGE: Mutex<HashMap<String, Activity>> = Mutex::new(HashMap::new());
    static ref RESOURCES_STORAGE: Mutex<HashMap<String, Resource>> = Mutex::new(HashMap::new());
    static ref EVENTS_STORAGE: Mutex<HashMap<String, Event>> = Mutex::new(HashMap::new());
    static ref ID_COUNTER: Mutex<u64> = Mutex::new(0);
}

#[derive(Clone, Debug, CandidType, Deserialize, Serialize)]
struct User {
    user_id: String,
    email: String,
    phone_number: String,
    role: Role,
}

#[derive(Clone, Debug, CandidType, Deserialize, Serialize)]
struct Plot {
    plot_id: String,
    owner_id: String,
    size: u32,
}

#[derive(Clone, Debug, CandidType, Deserialize, Serialize)]
struct Activity {
    activity_id: String,
    plot_id: String,
    description: String,
    timestamp: String,
}

#[derive(Clone, Debug, CandidType, Deserialize, Serialize)]
struct Resource {
    resource_id: String,
    name: String,
    quantity: u32,
}

#[derive(Clone, Debug, CandidType, Deserialize, Serialize)]
struct Event {
    event_id: String,
    name: String,
    date: String,
    location: String,
}

#[derive(Clone, Debug, CandidType, Deserialize, PartialEq)]
enum Role {
    Admin,
    User,
}

#[derive(Debug, CandidType, Deserialize, PartialEq)]
enum Error {
    NotFound(String),
    InvalidPayload(String),
    Unauthorized(String),
    InternalError(String),
}

#[derive(Debug, CandidType, Deserialize)]
enum Message {
    Success,
    Error(Error),
}

// Function to get the current timestamp
fn current_timestamp() -> String {
    let start = SystemTime::now();
    let since_the_epoch = start.duration_since(UNIX_EPOCH).expect("Time went backwards");
    format!("{}", since_the_epoch.as_secs())
}

// Function to validate email
fn validate_email(email: &str) -> bool {
    let email_regex = regex::Regex::new(r"^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$").unwrap();
    email_regex.is_match(email)
}

// Function to validate phone number
fn validate_phone_number(phone_number: &str) -> bool {
    let phone_regex = regex::Regex::new(r"^\+?[1-9]\d{1,14}$").unwrap();
    phone_regex.is_match(phone_number)
}

// Function to check if a user is an admin
fn is_admin(user_id: &str) -> bool {
    let storage = USERS_STORAGE.lock().unwrap();
    storage.iter().any(|(_, user)| user.user_id == user_id && user.role == Role::Admin)
}

// Function to get a new unique ID
fn get_new_id() -> u64 {
    let mut counter = ID_COUNTER.lock().unwrap();
    *counter += 1;
    *counter
}

// Function to create a new user
#[ic_cdk::update]
fn create_user(user_id: String, email: String, phone_number: String, role: Role) -> Result<Message, Message> {
    if !validate_email(&email) {
        return Err(Message::Error(Error::InvalidPayload("Invalid email format".to_string())));
    }
    if !validate_phone_number(&phone_number) {
        return Err(Message::Error(Error::InvalidPayload("Invalid phone number format".to_string())));
    }
    let user = User {
        user_id: user_id.clone(),
        email,
        phone_number,
        role,
    };
    let mut storage = USERS_STORAGE.lock().unwrap();
    storage.insert(user_id, user);
    Ok(Message::Success)
}

// Function to get all users
#[ic_cdk::query]
fn get_all_users() -> Vec<User> {
    let storage = USERS_STORAGE.lock().unwrap();
    storage.values().cloned().collect()
}

// Function to update a user
#[ic_cdk::update]
fn update_user(user_id: String, email: Option<String>, phone_number: Option<String>, role: Option<Role>, requester_id: String) -> Result<Message, Message> {
    if !is_admin(&requester_id) {
        return Err(Message::Error(Error::Unauthorized("Only admins can update users.".to_string())));
    }
    let mut storage = USERS_STORAGE.lock().unwrap();
    let user = storage.get_mut(&user_id);
    if let Some(user) = user {
        if let Some(email) = email {
            if !validate_email(&email) {
                return Err(Message::Error(Error::InvalidPayload("Invalid email format".to_string())));
            }
            user.email = email;
        }
        if let Some(phone_number) = phone_number {
            if !validate_phone_number(&phone_number) {
                return Err(Message::Error(Error::InvalidPayload("Invalid phone number format".to_string())));
            }
            user.phone_number = phone_number;
        }
        if let Some(role) = role {
            user.role = role;
        }
        Ok(Message::Success)
    } else {
        Err(Message::Error(Error::NotFound("User not found".to_string())))
    }
}

// Function to delete a user
#[ic_cdk::update]
fn delete_user(user_id: String, requester_id: String) -> Result<Message, Message> {
    if !is_admin(&requester_id) {
        return Err(Message::Error(Error::Unauthorized("Only admins can delete users.".to_string())));
    }
    let mut storage = USERS_STORAGE.lock().unwrap();
    if storage.remove(&user_id).is_some() {
        Ok(Message::Success)
    } else {
        Err(Message::Error(Error::NotFound("User not found".to_string())))
    }
}

// Function to create a new plot
#[ic_cdk::update]
fn create_plot(owner_id: String, size: u32) -> Result<Message, Message> {
    let plot = Plot {
        plot_id: get_new_id().to_string(),
        owner_id,
        size,
    };
    let mut storage = PLOTS_STORAGE.lock().unwrap();
    storage.insert(plot.plot_id.clone(), plot);
    Ok(Message::Success)
}

// Function to get all plots
#[ic_cdk::query]
fn get_all_plots() -> Vec<Plot> {
    let storage = PLOTS_STORAGE.lock().unwrap();
    storage.values().cloned().collect()
}

// Function to update a plot
#[ic_cdk::update]
fn update_plot(plot_id: String, owner_id: Option<String>, size: Option<u32>, requester_id: String) -> Result<Message, Message> {
    if !is_admin(&requester_id) {
        return Err(Message::Error(Error::Unauthorized("Only admins can update plots.".to_string())));
    }
    let mut storage = PLOTS_STORAGE.lock().unwrap();
    let plot = storage.get_mut(&plot_id);
    if let Some(plot) = plot {
        if let Some(owner_id) = owner_id {
            plot.owner_id = owner_id;
        }
        if let Some(size) = size {
            plot.size = size;
        }
        Ok(Message::Success)
    } else {
        Err(Message::Error(Error::NotFound("Plot not found".to_string())))
    }
}

// Function to delete a plot
#[ic_cdk::update]
fn delete_plot(plot_id: String, requester_id: String) -> Result<Message, Message> {
    if !is_admin(&requester_id) {
        return Err(Message::Error(Error::Unauthorized("Only admins can delete plots.".to_string())));
    }
    let mut storage = PLOTS_STORAGE.lock().unwrap();
    if storage.remove(&plot_id).is_some() {
        Ok(Message::Success)
    } else {
        Err(Message::Error(Error::NotFound("Plot not found".to_string())))
    }
}

// Function to create a new activity
#[ic_cdk::update]
fn create_activity(plot_id: String, description: String) -> Result<Message, Message> {
    let activity = Activity {
        activity_id: get_new_id().to_string(),
        plot_id,
        description,
        timestamp: current_timestamp(),
    };
    let mut storage = ACTIVITIES_STORAGE.lock().unwrap();
    storage.insert(activity.activity_id.clone(), activity);
    Ok(Message::Success)
}

// Function to get all activities
#[ic_cdk::query]
fn get_all_activities() -> Vec<Activity> {
    let storage = ACTIVITIES_STORAGE.lock().unwrap();
    storage.values().cloned().collect()
}

// Function to update an activity
#[ic_cdk::update]
fn update_activity(activity_id: String, plot_id: Option<String>, description: Option<String>, requester_id: String) -> Result<Message
// Function to update an activity
#[ic_cdk::update]
fn update_activity(activity_id: String, plot_id: Option<String>, description: Option<String>, requester_id: String) -> Result<Message, Message> {
    if !is_admin(&requester_id) {
        return Err(Message::Error(Error::Unauthorized("Only admins can update activities.".to_string())));
    }
    let mut storage = ACTIVITIES_STORAGE.lock().unwrap();
    let activity = storage.get_mut(&activity_id);
    if let Some(activity) = activity {
        if let Some(plot_id) = plot_id {
            activity.plot_id = plot_id;
        }
        if let Some(description) = description {
            activity.description = description;
        }
        Ok(Message::Success)
    } else {
        Err(Message::Error(Error::NotFound("Activity not found".to_string())))
    }
}

// Function to delete an activity
#[ic_cdk::update]
fn delete_activity(activity_id: String, requester_id: String) -> Result<Message, Message> {
    if !is_admin(&requester_id) {
        return Err(Message::Error(Error::Unauthorized("Only admins can delete activities.".to_string())));
    }
    let mut storage = ACTIVITIES_STORAGE.lock().unwrap();
    if storage.remove(&activity_id).is_some() {
        Ok(Message::Success)
    } else {
        Err(Message::Error(Error::NotFound("Activity not found".to_string())))
    }
}

// Function to create a new resource
#[ic_cdk::update]
fn create_resource(name: String, quantity: u32) -> Result<Message, Message> {
    let resource = Resource {
        resource_id: get_new_id().to_string(),
        name,
        quantity,
    };
    let mut storage = RESOURCES_STORAGE.lock().unwrap();
    storage.insert(resource.resource_id.clone(), resource);
    Ok(Message::Success)
}

// Function to get all resources
#[ic_cdk::query]
fn get_all_resources() -> Vec<Resource> {
    let storage = RESOURCES_STORAGE.lock().unwrap();
    storage.values().cloned().collect()
}

// Function to update a resource
#[ic_cdk::update]
fn update_resource(resource_id: String, name: Option<String>, quantity: Option<u32>, requester_id: String) -> Result<Message, Message> {
    if !is_admin(&requester_id) {
        return Err(Message::Error(Error::Unauthorized("Only admins can update resources.".to_string())));
    }
    let mut storage = RESOURCES_STORAGE.lock().unwrap();
    let resource = storage.get_mut(&resource_id);
    if let Some(resource) = resource {
        if let Some(name) = name {
            resource.name = name;
        }
        if let Some(quantity) = quantity {
            resource.quantity = quantity;
        }
        Ok(Message::Success)
    } else {
        Err(Message::Error(Error::NotFound("Resource not found".to_string())))
    }
}

// Function to delete a resource
#[ic_cdk::update]
fn delete_resource(resource_id: String, requester_id: String) -> Result<Message, Message> {
    if !is_admin(&requester_id) {
        return Err(Message::Error(Error::Unauthorized("Only admins can delete resources.".to_string())));
    }
    let mut storage = RESOURCES_STORAGE.lock().unwrap();
    if storage.remove(&resource_id).is_some() {
        Ok(Message::Success)
    } else {
        Err(Message::Error(Error::NotFound("Resource not found".to_string())))
    }
}

// Function to create a new event
#[ic_cdk::update]
fn create_event(name: String, date: String, location: String) -> Result<Message, Message> {
    let event = Event {
        event_id: get_new_id().to_string(),
        name,
        date,
        location,
    };
    let mut storage = EVENTS_STORAGE.lock().unwrap();
    storage.insert(event.event_id.clone(), event);
    Ok(Message::Success)
}

// Function to get all events
#[ic_cdk::query]
fn get_all_events() -> Vec<Event> {
    let storage = EVENTS_STORAGE.lock().unwrap();
    storage.values().cloned().collect()
}

// Function to update an event
#[ic_cdk::update]
fn update_event(event_id: String, name: Option<String>, date: Option<String>, location: Option<String>, requester_id: String) -> Result<Message, Message> {
    if !is_admin(&requester_id) {
        return Err(Message::Error(Error::Unauthorized("Only admins can update events.".to_string())));
    }
    let mut storage = EVENTS_STORAGE.lock().unwrap();
    let event = storage.get_mut(&event_id);
    if let Some(event) = event {
        if let Some(name) = name {
            event.name = name;
        }
        if let Some(date) = date {
            event.date = date;
        }
        if let Some(location) = location {
            event.location = location;
        }
        Ok(Message::Success)
    } else {
        Err(Message::Error(Error::NotFound("Event not found".to_string())))
    }
}

// Function to delete an event
#[ic_cdk::update]
fn delete_event(event_id: String, requester_id: String) -> Result<Message, Message> {
    if !is_admin(&requester_id) {
        return Err(Message::Error(Error::Unauthorized("Only admins can delete events.".to_string())));
    }
    let mut storage = EVENTS_STORAGE.lock().unwrap();
    if storage.remove(&event_id).is_some() {
        Ok(Message::Success)
    } else {
        Err(Message::Error(Error::NotFound("Event not found".to_string())))
    }
}

// Export the Candid interface for this canister
#[export_candid]
fn export_candid() -> String {
    ic_cdk::export::candid::export_service!();
    __export_service()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_user() {
        let result = create_user("1".to_string(), "test@example.com".to_string(), "+1234567890".to_string(), Role::User);
        assert!(matches!(result, Ok(Message::Success)));
    }

    #[test]
    fn test_invalid_email() {
        let result = create_user("2".to_string(), "invalid-email".to_string(), "+1234567890".to_string(), Role::User);
        assert!(matches!(result, Err(Message::Error(Error::InvalidPayload(_)))));
    }

    #[test]
    fn test_invalid_phone() {
        let result = create_user("3".to_string(), "test@example.com".to_string(), "invalid-phone".to_string(), Role::User);
        assert!(matches!(result, Err(Message::Error(Error::InvalidPayload(_)))));
    }
}

ic_cdk::export_candid!();
