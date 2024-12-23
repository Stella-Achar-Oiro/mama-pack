
#[macro_use]
extern crate serde;
use candid::{Decode, Encode};
use ic_cdk::api::time;
use ic_stable_structures::memory_manager::{MemoryId, MemoryManager, VirtualMemory};
use ic_stable_structures::{BoundedStorable, Cell, DefaultMemoryImpl, StableBTreeMap, Storable};
use std::{borrow::Cow, cell::RefCell};

// Define memory and storage types
type Memory = VirtualMemory<DefaultMemoryImpl>;
type IdCell = Cell<u64, Memory>;

// Pregnancy Stage enum for tracking progress
#[derive(candid::CandidType, Clone, Serialize, Deserialize)]
enum PregnancyStage {
    FirstTrimester,
    SecondTrimester,
    ThirdTrimester,
    PostPartum,
}

impl Default for PregnancyStage {
    fn default() -> Self {
        PregnancyStage::FirstTrimester
    }
}

// Health Status enum
#[derive(candid::CandidType, Clone, Serialize, Deserialize)]
enum HealthStatus {
    Normal,
    NeedsAttention,
    Critical,
}

impl Default for HealthStatus {
    fn default() -> Self {
        HealthStatus::Normal
    }
} 

// Mother's profile with essential health information
#[derive(candid::CandidType, Clone, Serialize, Deserialize, Default)]
struct MotherProfile {
    id: u64,
    name: String,
    age: u8,
    blood_type: String,
    expected_delivery_date: u64,
    stage: PregnancyStage,
    health_status: HealthStatus,
    created_at: u64,
    last_checkup: u64,
    medical_history: Vec<String>,
    emergency_contact: String,
}

// Health Record for tracking checkups and vitals
#[derive(candid::CandidType, Clone, Serialize, Deserialize)]
struct HealthRecord {
    id: u64,
    mother_id: u64,
    date: u64,
    blood_pressure: String,
    weight: f32,
    symptoms: Vec<String>,
    notes: String,
    next_appointment: u64,
    health_status: HealthStatus,
}

// Payload for creating/updating mother's profile
#[derive(candid::CandidType, Serialize, Deserialize)]
struct MotherProfilePayload {
    name: String,
    age: u8,
    blood_type: String,
    expected_delivery_date: u64,
    medical_history: Vec<String>,
    emergency_contact: String,
}

// Payload for health record entry
#[derive(candid::CandidType, Serialize, Deserialize)]
struct HealthRecordPayload {
    mother_id: u64,
    blood_pressure: String,
    weight: f32,
    symptoms: Vec<String>,
    notes: String,
    next_appointment: u64,
}

// Implement Storable for MotherProfile
impl Storable for MotherProfile {
    fn to_bytes(&self) -> std::borrow::Cow<[u8]> {
        Cow::Owned(Encode!(self).unwrap())
    }

    fn from_bytes(bytes: std::borrow::Cow<[u8]>) -> Self {
        Decode!(bytes.as_ref(), Self).unwrap()
    }
}

// Implement BoundedStorable for MotherProfile
impl BoundedStorable for MotherProfile {
    const MAX_SIZE: u32 = 2048;
    const IS_FIXED_SIZE: bool = false;
}

// Implement Storable for HealthRecord
impl Storable for HealthRecord {
    fn to_bytes(&self) -> std::borrow::Cow<[u8]> {
        Cow::Owned(Encode!(self).unwrap())
    }

    fn from_bytes(bytes: std::borrow::Cow<[u8]>) -> Self {
        Decode!(bytes.as_ref(), Self).unwrap()
    }
}

// Implement BoundedStorable for HealthRecord
impl BoundedStorable for HealthRecord {
    const MAX_SIZE: u32 = 2048;
    const IS_FIXED_SIZE: bool = false;
}

// Thread local storage
thread_local! {
    static MEMORY_MANAGER: RefCell<MemoryManager<DefaultMemoryImpl>> = RefCell::new(
        MemoryManager::init(DefaultMemoryImpl::default())
    );

    static ID_COUNTER: RefCell<IdCell> = RefCell::new(
        IdCell::init(MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(0))), 0)
            .expect("Cannot create id counter")
    );

    static PROFILE_STORAGE: RefCell<StableBTreeMap<u64, MotherProfile, Memory>> = RefCell::new(
        StableBTreeMap::init(MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(1))))
    );

    static HEALTH_RECORD_STORAGE: RefCell<StableBTreeMap<u64, HealthRecord, Memory>> = RefCell::new(
        StableBTreeMap::init(MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(2))))
    );
}

// Error handling
#[derive(candid::CandidType, Deserialize, Serialize)]
enum Error {
    NotFound { msg: String },
    InvalidInput { msg: String },
    SystemError { msg: String },
}

// Helper function to determine pregnancy stage based on EDD
fn calculate_pregnancy_stage(edd: u64) -> PregnancyStage {
    let now = time();
    let time_diff = if edd > now {
        edd - now
    } else {
        0
    };
    
    let weeks_to_edd = time_diff / (7 * 24 * 60 * 60 * 1_000_000_000);
    
    match weeks_to_edd {
        0..=12 => PregnancyStage::ThirdTrimester,
        13..=26 => PregnancyStage::SecondTrimester,
        27..=40 => PregnancyStage::FirstTrimester,
        _ => PregnancyStage::PostPartum,
    }
}

// Create new mother profile
#[ic_cdk::update]
fn create_mother_profile(payload: MotherProfilePayload) -> Result<MotherProfile, Error> {
    if payload.age < 13 || payload.age > 65 {
        return Err(Error::InvalidInput {
            msg: "Invalid age range".to_string(),
        });
    }

    let id = ID_COUNTER
        .with(|counter| {
            let current_value = *counter.borrow().get();
            counter.borrow_mut().set(current_value + 1)
        })
        .expect("Cannot increment id counter");

    let stage = calculate_pregnancy_stage(payload.expected_delivery_date);
    
    let profile = MotherProfile {
        id,
        name: payload.name,
        age: payload.age,
        blood_type: payload.blood_type,
        expected_delivery_date: payload.expected_delivery_date,
        stage,
        health_status: HealthStatus::Normal,
        created_at: time(),
        last_checkup: time(),
        medical_history: payload.medical_history,
        emergency_contact: payload.emergency_contact,
    };

    PROFILE_STORAGE.with(|storage| storage.borrow_mut().insert(id, profile.clone()));
    Ok(profile)
}

// Add health record
#[ic_cdk::update]
fn add_health_record(payload: HealthRecordPayload) -> Result<HealthRecord, Error> {
    // Verify mother exists
    PROFILE_STORAGE.with(|storage| {
        if !storage.borrow().contains_key(&payload.mother_id) {
            return Err(Error::NotFound {
                msg: format!("Mother with id={} not found", payload.mother_id),
            });
        }
        Ok(())
    })?;

    let id = ID_COUNTER
        .with(|counter| {
            let current_value = *counter.borrow().get();
            counter.borrow_mut().set(current_value + 1)
        })
        .expect("Cannot increment id counter");

    // Determine health status based on symptoms and vitals
    let health_status = analyze_health_status(&payload);

    let record = HealthRecord {
    id,
    mother_id: payload.mother_id,
    date: time(),
    blood_pressure: payload.blood_pressure,
    weight: payload.weight,
    symptoms: payload.symptoms,
    notes: payload.notes,
    next_appointment: payload.next_appointment,
    health_status: health_status.clone(), // Add .clone() here
    };

    // Update mother's profile with latest checkup and health status
    update_mother_status(payload.mother_id, &health_status)?;

    HEALTH_RECORD_STORAGE.with(|storage| storage.borrow_mut().insert(id, record.clone()));
    Ok(record)
}

// Helper function to analyze health status based on symptoms and vitals
fn analyze_health_status(record: &HealthRecordPayload) -> HealthStatus {
    let has_critical_symptoms = record.symptoms.iter().any(|s| 
        s.contains("severe") || 
        s.contains("emergency") || 
        s.contains("critical")
    );

    if has_critical_symptoms {
        HealthStatus::Critical
    } else if !record.symptoms.is_empty() {
        HealthStatus::NeedsAttention
    } else {
        HealthStatus::Normal
    }
}

// Update mother's status based on health record
fn update_mother_status(mother_id: u64, health_status: &HealthStatus) -> Result<(), Error> {
    PROFILE_STORAGE.with(|storage| {
        let mut storage = storage.borrow_mut();
        match storage.get(&mother_id) {
            Some(mut profile) => {
                profile.health_status = health_status.clone();
                profile.last_checkup = time();
                storage.insert(mother_id, profile);
                Ok(())
            }
            None => Err(Error::NotFound {
                msg: format!("Mother with id={} not found", mother_id),
            }),
        }
    })
}

// Get mother's profile
#[ic_cdk::query]
fn get_mother_profile(id: u64) -> Result<MotherProfile, Error> {
    PROFILE_STORAGE.with(|storage| {
        match storage.borrow().get(&id) {
            Some(profile) => Ok(profile),
            None => Err(Error::NotFound {
                msg: format!("Mother with id={} not found", id),
            }),
        }
    })
}

// Get mother's health records
#[ic_cdk::query]
fn get_mother_health_records(mother_id: u64) -> Result<Vec<HealthRecord>, Error> {
    let records = HEALTH_RECORD_STORAGE.with(|storage| {
        storage
            .borrow()
            .iter()
            .filter(|(_, record)| record.mother_id == mother_id)
            .map(|(_, record)| record.clone())
            .collect::<Vec<HealthRecord>>()
    });

    if records.is_empty() {
        Err(Error::NotFound {
            msg: format!("No health records found for mother_id={}", mother_id),
        })
    } else {
        Ok(records)
    }
}

// Get high-risk profiles
#[ic_cdk::query]
fn get_high_risk_profiles() -> Vec<MotherProfile> {
    PROFILE_STORAGE.with(|storage| {
        storage
            .borrow()
            .iter()
            .filter(|(_, profile)| matches!(profile.health_status, HealthStatus::Critical))
            .map(|(_, profile)| profile.clone())
            .collect()
    })
}

// Export Candid interface
ic_cdk::export_candid!();