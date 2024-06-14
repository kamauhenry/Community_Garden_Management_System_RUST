#[macro_use]
extern crate serde;
use candid::{Decode, Encode};
use ic_cdk::api::time;
use ic_stable_structures::memory_manager::{MemoryId, MemoryManager, VirtualMemory};
use ic_stable_structures::{BoundedStorable, Cell, DefaultMemoryImpl, StableBTreeMap, Storable};
use regex::Regex;
use std::{borrow::Cow, cell::RefCell};

type Memory = VirtualMemory<DefaultMemoryImpl>;
type IdCell = Cell<u64, Memory>;

#[derive(candid::CandidType, Clone, Serialize, Deserialize, Default)]
struct User {
    user_id: String,
    owner: String,
    name: String,
    email: String,
    phone_number: String,
    created_at: String,
}

#[derive(candid::CandidType, Clone, Serialize, Deserialize, Default)]
struct Plot {
    id: String,
    user_id: String,
    size: String,
    location: String,
    reserved_until: String,
    created_at: String,
}

#[derive(candid::CandidType, Clone, Serialize, Deserialize, Default)]
struct Activity {
    id: String,
    plot_id: String,
    description: String,
    date: String,
    created_at: String,
}

#[derive(candid::CandidType, Clone, Serialize, Deserialize, Default)]
struct Resource {
    id: String,
    name: String,
    quantity: u64,
    available: bool,
    created_at: String,
}

#[derive(candid::CandidType, Clone, Serialize, Deserialize, Default)]
struct Event {
    id: String,
    title: String,
    description: String,
    date: String,
    location: String,
    created_at: String,
}

impl Storable for User {
    fn to_bytes(&self) -> Cow<[u8]> {
        Cow::Owned(Encode!(self).unwrap())
    }

    fn from_bytes(bytes: Cow<[u8]>) -> Self {
        Decode!(bytes.as_ref(), Self).unwrap()
    }
}

impl BoundedStorable for User {
    const MAX_SIZE: u32 = 512;
    const IS_FIXED_SIZE: bool = false;
}

impl Storable for Plot {
    fn to_bytes(&self) -> Cow<[u8]> {
        Cow::Owned(Encode!(self).unwrap())
    }

    fn from_bytes(bytes: Cow<[u8]>) -> Self {
        Decode!(bytes.as_ref(), Self).unwrap()
    }
}

impl BoundedStorable for Plot {
    const MAX_SIZE: u32 = 512;
    const IS_FIXED_SIZE: bool = false;
}

impl Storable for Activity {
    fn to_bytes(&self) -> Cow<[u8]> {
        Cow::Owned(Encode!(self).unwrap())
    }

    fn from_bytes(bytes: Cow<[u8]>) -> Self {
        Decode!(bytes.as_ref(), Self).unwrap()
    }
}

impl BoundedStorable for Activity {
    const MAX_SIZE: u32 = 512;
    const IS_FIXED_SIZE: bool = false;
}

impl Storable for Resource {
    fn to_bytes(&self) -> Cow<[u8]> {
        Cow::Owned(Encode!(self).unwrap())
    }

    fn from_bytes(bytes: Cow<[u8]>) -> Self {
        Decode!(bytes.as_ref(), Self).unwrap()
    }
}

impl BoundedStorable for Resource {
    const MAX_SIZE: u32 = 512;
    const IS_FIXED_SIZE: bool = false;
}

impl Storable for Event {
    fn to_bytes(&self) -> Cow<[u8]> {
        Cow::Owned(Encode!(self).unwrap())
    }

    fn from_bytes(bytes: Cow<[u8]>) -> Self {
        Decode!(bytes.as_ref(), Self).unwrap()
    }
}

impl BoundedStorable for Event {
    const MAX_SIZE: u32 = 512;
    const IS_FIXED_SIZE: bool = false;
}

thread_local! {
    static MEMORY_MANAGER: RefCell<MemoryManager<DefaultMemoryImpl>> = RefCell::new(
        MemoryManager::init(DefaultMemoryImpl::default())
    );

    static ID_COUNTER: RefCell<IdCell> = RefCell::new(
        IdCell::init(MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(0))), 0)
            .expect("Cannot create a counter")
    );

    static USERS_STORAGE: RefCell<StableBTreeMap<u64, User, Memory>> =
        RefCell::new(StableBTreeMap::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(1)))
    ));

    static PLOTS_STORAGE: RefCell<StableBTreeMap<u64, Plot, Memory>> =
        RefCell::new(StableBTreeMap::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(2)))
    ));

    static ACTIVITIES_STORAGE: RefCell<StableBTreeMap<u64, Activity, Memory>> =
        RefCell::new(StableBTreeMap::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(3)))
    ));

    static RESOURCES_STORAGE: RefCell<StableBTreeMap<u64, Resource, Memory>> =
        RefCell::new(StableBTreeMap::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(4)))
    ));

    static EVENTS_STORAGE: RefCell<StableBTreeMap<u64, Event, Memory>> =
        RefCell::new(StableBTreeMap::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(5)))
    ));
}

#[derive(candid::CandidType, Deserialize, Serialize)]
struct UserPayload {
    name: String,
    email: String,
    phone_number: String,
}

#[derive(candid::CandidType, Deserialize, Serialize)]
struct PlotPayload {
    user_id: String,
    size: String,
    location: String,
    reserved_until: String,
}

#[derive(candid::CandidType, Deserialize, Serialize)]
struct ActivityPayload {
    plot_id: String,
    description: String,
    date: String,
}

#[derive(candid::CandidType, Deserialize, Serialize)]
struct ResourcePayload {
    name: String,
    quantity: u64,
    available: bool,
}

#[derive(candid::CandidType, Deserialize, Serialize)]
struct EventPayload {
    title: String,
    description: String,
    date: String,
    location: String,
}

#[derive(candid::CandidType, Deserialize, Serialize)]
enum Message {
    Success(String),
    Error(String),
    NotFound(String),
    InvalidPayload(String),
}

// Function to create a new user profile
#[ic_cdk::update]
fn create_user_profile(payload: UserPayload) -> Result<User, Message> {
    if payload.name.is_empty() || payload.email.is_empty() || payload.phone_number.is_empty() {
        return Err(Message::InvalidPayload(
            "Ensure 'name', 'email', and 'phone_number' are provided.".to_string(),
        ));
    }
    
    // Validate email
    let email_regex = Regex::new(r"^[^\s@]+@[^\s@]+\.[^\s@]+$").unwrap();
    if !email_regex.is_match(&payload.email) {
        return Err(Message::InvalidPayload(
            "Invalid email format: Ensure the email is in the correct format.".to_string(),
        ));
    }

    // Validate the email address to make it unique
    let email = payload.email.clone();
    let email_exists = USERS_STORAGE.with(|storage| {
        storage.borrow().iter().any(|(_, user)| user.email == email)
    });
    if email_exists {
        return Err(Message::InvalidPayload(
            "Email already exists: Ensure the email address is unique.".to_string(),
        ));
    }
    
    // Validate phone number
    let phone_number_regex = Regex::new(r"^\d{10}$").unwrap();
    if !phone_number_regex.is_match(&payload.phone_number) {
        return Err(Message::InvalidPayload(
            "Invalid phone number: Ensure the phone number is in the correct format.".to_string(),
        ));
    }

    let id = ID_COUNTER
        .with(|counter| {
            let current_value = *counter.borrow().get();
            counter.borrow_mut().set(current_value + 1)
        })
        .expect("Cannot increment ID counter");

    let user = User {
        user_id: id.to_string(),
        owner: ic_cdk::caller().to_string(),
        name: payload.name,
        email: payload.email,
        phone_number: payload.phone_number,
        created_at: time().to_string(),
    };

    USERS_STORAGE.with(|storage| storage.borrow_mut().insert(id, user.clone()));

    Ok(user)
}

// Function to update a user profile
#[ic_cdk::update]
fn update_user_profile(user_id: String, payload: UserPayload) -> Result<User, Message> {
    if payload.name.is_empty() || payload.email.is_empty() || payload.phone_number.is_empty() {
        return Err(Message::InvalidPayload(
            "Ensure 'name', 'email', and 'phone_number' are provided.".to_string(),
        ));
    }

    // Validate email
    let email_regex = Regex::new(r"^[^\s@]+@[^\s@]+\.[^\s@]+$").unwrap();
    if !email_regex.is_match(&payload.email) {
        return Err(Message::InvalidPayload(
            "Invalid email format: Ensure the email is in the correct format.".to_string(),
        ));
    }

    // Validate the email address to make it unique
    let email = payload.email.clone();
    let email_exists = USERS_STORAGE.with(|storage| {
        storage.borrow().iter().any(|(_, user)| user.email == email)
    });
    if email_exists {
        return Err(Message::InvalidPayload(
            "Email already exists: Ensure the email address is unique.".to_string(),
        ));
    }

    // Validate phone number
    let phone_number_regex = Regex::new(r"^\d{10}$").unwrap();
    if !phone_number_regex.is_match(&payload.phone_number) {
        return Err(Message::InvalidPayload(
            "Invalid phone number: Ensure the phone number is in the correct format.".to_string(),
        ));
    }

    let user = USERS_STORAGE.with(|storage| {
        let user = storage.borrow().iter().find(|(_, user)| user.user_id == user_id);
        match user {
            Some((id, _)) => {
                let updated_user = User {
                    user_id: user_id,
                    owner: ic_cdk::caller().to_string(),
                    name: payload.name,
                    email: payload.email,
                    phone_number: payload.phone_number,
                    created_at: time().to_string(),
                };
                storage.borrow_mut().insert(id, updated_user.clone());
                Ok(updated_user)
            }
            None => Err(Message::NotFound("User not found.".to_string())),
        }
    });

    user
}

// Function to get a user profile id
#[ic_cdk::query]
fn get_user_profile(user_id: String) -> Result<User, Message> {
    USERS_STORAGE.with(|storage| {
        let user = storage.borrow().iter().find(|(_, user)| user.user_id == user_id);
        match user {
            Some((_, record)) => Ok(record.clone()),
            None => Err(Message::NotFound("User not found.".to_string())),
        }
    })
}

// Function to get all users
#[ic_cdk::query]
fn get_all_users() -> Result<Vec<User>, Message> {
    USERS_STORAGE.with(|storage| {
        let records: Vec<User> = storage.borrow().iter().map(|(_, record)| record.clone()).collect();
        if records.is_empty() {
            Err(Message::NotFound("No users found.".to_string()))
        } else {
            Ok(records)
        }
    })
}

// Function to create a new plot
#[ic_cdk::update]
fn create_plot(payload: PlotPayload) -> Result<Plot, Message> {
    if payload.user_id.is_empty() || payload.size.is_empty() || payload.location.is_empty() {
        return Err(Message::InvalidPayload(
            "Ensure 'user_id', 'size', and 'location' are provided.".to_string(),
        ));
    }

    // Validate the user ID
    let user_id = payload.user_id.clone();
    let user_exists = USERS_STORAGE.with(|storage| {
        storage.borrow().iter().any(|(_, user)| user.user_id == user_id)
    });
    if !user_exists {
        return Err(Message::InvalidPayload(
            "User not found: Ensure the user ID is correct.".to_string(),
        ));
    }

    let id = ID_COUNTER
        .with(|counter| {
            let current_value = *counter.borrow().get();
            counter.borrow_mut().set(current_value + 1)
        })
        .expect("Cannot increment ID counter");

    let plot = Plot {
        id: id.to_string(),
        user_id: payload.user_id,
        size: payload.size,
        location: payload.location,
        reserved_until: payload.reserved_until,
        created_at: time().to_string(),
    };

    PLOTS_STORAGE.with(|storage| storage.borrow_mut().insert(id, plot.clone()));

    Ok(plot)
}

// Function to get all plots
#[ic_cdk::query]
fn get_all_plots() -> Result<Vec<Plot>, Message> {
    PLOTS_STORAGE.with(|storage| {
        let records: Vec<Plot> = storage.borrow().iter().map(|(_, record)| record.clone()).collect();
        if records.is_empty() {
            Err(Message::NotFound("No plots found.".to_string()))
        } else {
            Ok(records)
        }
    })
}

// Function to create a new activity
#[ic_cdk::update]
fn create_activity(payload: ActivityPayload) -> Result<Activity, Message> {
    if payload.plot_id.is_empty() || payload.description.is_empty() || payload.date.is_empty() {
        return Err(Message::InvalidPayload(
            "Ensure 'plot_id', 'description', and 'date' are provided.".to_string(),
        ));
    }

    // Validate the plot ID
    let plot_id = payload.plot_id.clone();
    let plot_exists = PLOTS_STORAGE.with(|storage| {
        storage.borrow().iter().any(|(_, plot)| plot.id == plot_id)
    });
    if !plot_exists {
        return Err(Message::InvalidPayload(
            "Plot not found: Ensure the plot ID is correct.".to_string(),
        ));
    }

    let id = ID_COUNTER
        .with(|counter| {
            let current_value = *counter.borrow().get();
            counter.borrow_mut().set(current_value + 1)
        })
        .expect("Cannot increment ID counter");

    let activity = Activity {
        id: id.to_string(),
        plot_id: payload.plot_id,
        description: payload.description,
        date: payload.date,
        created_at: time().to_string(),
    };

    ACTIVITIES_STORAGE.with(|storage| storage.borrow_mut().insert(id, activity.clone()));

    Ok(activity)
}

// Function to get all activities
#[ic_cdk::query]
fn get_all_activities() -> Result<Vec<Activity>, Message> {
    ACTIVITIES_STORAGE.with(|storage| {
        let records: Vec<Activity> = storage.borrow().iter().map(|(_, record)| record.clone()).collect();
        if records.is_empty() {
            Err(Message::NotFound("No activities found.".to_string()))
        } else {
            Ok(records)
        }
    })
}

// Function to create a new resource
#[ic_cdk::update]
fn create_resource(payload: ResourcePayload) -> Result<Resource, Message> {
    if payload.name.is_empty() || payload.quantity == 0 {
        return Err(Message::InvalidPayload(
            "Ensure 'name' and 'quantity' are provided.".to_string(),
        ));
    }

    let id = ID_COUNTER
        .with(|counter| {
            let current_value = *counter.borrow().get();
            counter.borrow_mut().set(current_value + 1)
        })
        .expect("Cannot increment ID counter");

    let resource = Resource {
        id: id.to_string(),
        name: payload.name,
        quantity: payload.quantity,
        available: payload.available,
        created_at: time().to_string(),
    };

    RESOURCES_STORAGE.with(|storage| storage.borrow_mut().insert(id, resource.clone()));

    Ok(resource)
}

// Function to get all resources
#[ic_cdk::query]
fn get_all_resources() -> Result<Vec<Resource>, Message> {
    RESOURCES_STORAGE.with(|storage| {
        let records: Vec<Resource> = storage.borrow().iter().map(|(_, record)| record.clone()).collect();
        if records.is_empty() {
            Err(Message::NotFound("No resources found.".to_string()))
        } else {
            Ok(records)
        }
    })
}

// Function to create a new event
#[ic_cdk::update]
fn create_event(payload: EventPayload) -> Result<Event, Message> {
    if payload.title.is_empty() || payload.description.is_empty() || payload.date.is_empty() || payload.location.is_empty() {
        return Err(Message::InvalidPayload(
            "Ensure 'title', 'description', 'date', and 'location' are provided.".to_string(),
        ));
    }

    let id = ID_COUNTER
        .with(|counter| {
            let current_value = *counter.borrow().get();
            counter.borrow_mut().set(current_value + 1)
        })
        .expect("Cannot increment ID counter");

    let event = Event {
        id: id.to_string(),
        title: payload.title,
        description: payload.description,
        date: payload.date,
        location: payload.location,
        created_at: time().to_string(),
    };

    EVENTS_STORAGE.with(|storage| storage.borrow_mut().insert(id, event.clone()));

    Ok(event)
}

// Function to get all events
#[ic_cdk::query]
fn get_all_events() -> Result<Vec<Event>, Message> {
    EVENTS_STORAGE.with(|storage| {
        let records: Vec<Event> = storage.borrow().iter().map(|(_, record)| record.clone()).collect();
        if records.is_empty() {
            Err(Message::NotFound("No events found.".to_string()))
        } else {
            Ok(records)
        }
    })
}

// Error types
#[derive(candid::CandidType, Deserialize, Serialize)]
enum Error {
    NotFound { msg: String },
}

// need this to generate candid
ic_cdk::export_candid!();
