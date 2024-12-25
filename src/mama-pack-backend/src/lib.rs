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
    AuthorizationError { msg: String },
    ValidationError { msg: String },
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
        0 => PregnancyStage::PostPartum,
        1..=13 => PregnancyStage::ThirdTrimester,
        14..=27 => PregnancyStage::SecondTrimester,
        _ => PregnancyStage::FirstTrimester,
    }
}
//Helper functions for code maintanability and reusability

//Generate Unique ID
fn generate_new_id() -> Result<u64, Error> {
    ID_COUNTER.with(|counter| {
        let current_value = *counter.borrow().get();
        counter
            .borrow_mut()
            .set(current_value + 1)
            .map_err(|_| Error::SystemError { msg: "Failed to increment ID counter".to_string() })
    })
}
//END OF Helper Functions 

// Create new mother profile
#[ic_cdk::update]
fn create_mother_profile(payload: MotherProfilePayload) -> Result<MotherProfile, Error> {
    // Validate the payload first
    validate_mother_profile(&payload)?;

    let id = generate_new_id()?;

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

    let id = generate_new_id()?;

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
    // Parse blood pressure
    let bp_parts: Vec<&str> = record.blood_pressure.split('/').collect();
    if bp_parts.len() == 2 {
        if let (Ok(systolic), Ok(diastolic)) = (
            bp_parts[0].trim().parse::<i32>(),
            bp_parts[1].trim().parse::<i32>()
        ) {
            // Check for concerning blood pressure
            if systolic >= 140 || diastolic >= 90 || systolic < 90 || diastolic < 60 {
                return HealthStatus::Critical;
            }
        }
    }

    // Check weight changes
    if record.weight < 45.0 || record.weight > 100.0 {
        return HealthStatus::NeedsAttention;
    }

    // Check symptoms
    let critical_symptoms = [
        "severe", "emergency", "critical", "bleeding",
        "seizure", "unconscious", "fever", "headache"
    ];
    
    let concerning_symptoms = [
        "nausea", "vomiting", "swelling", "pain",
        "discomfort", "fatigue", "dizziness"
    ];

    if record.symptoms.iter().any(|s| 
        critical_symptoms.iter().any(|cs| s.to_lowercase().contains(cs))
    ) {
        HealthStatus::Critical
    } else if record.symptoms.iter().any(|s|
        concerning_symptoms.iter().any(|cs| s.to_lowercase().contains(cs))
    ) {
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

// Get critical cases
#[ic_cdk::query]
fn get_critical_cases() -> Vec<MotherProfile> {
    PROFILE_STORAGE.with(|storage| {
        storage
            .borrow()
            .iter()
            .filter(|(_, profile)| matches!(profile.health_status, HealthStatus::Critical))
            .map(|(_, profile)| profile.clone())
            .collect()
    })
}

// Get upcoming appointments
#[ic_cdk::query]
fn get_upcoming_appointments(days: u64) -> Vec<(MotherProfile, HealthRecord)> {
    let now = time();
    let target = now + (days * 24 * 60 * 60 * 1_000_000_000);
    
    HEALTH_RECORD_STORAGE.with(|record_storage| {
        PROFILE_STORAGE.with(|profile_storage| {
            let records = record_storage.borrow();
            let profiles = profile_storage.borrow();
            
            records
                .iter()
                .filter(|(_, record)| {
                    record.next_appointment > now && record.next_appointment <= target
                })
                .filter_map(|(_, record)| {
                    profiles
                        .get(&record.mother_id)
                        .map(|profile| (profile.clone(), record.clone()))
                })
                .collect()
        })
    })
}

// Export Candid interface
ic_cdk::export_candid!();

fn validate_mother_profile(payload: &MotherProfilePayload) -> Result<(), Error> {
    // Validate age
    if payload.age < 13 || payload.age > 65 {
        return Err(Error::InvalidInput {
            msg: "Invalid age range. Must be between 13 and 65".to_string(),
        });
    }

    // Validate blood type
    let valid_blood_types = ["A+", "A-", "B+", "B-", "AB+", "AB-", "O+", "O-"];
    if !valid_blood_types.contains(&payload.blood_type.as_str()) {
        return Err(Error::InvalidInput {
            msg: "Invalid blood type".to_string(),
        });
    }

    // Validate expected delivery date
    let now = time();
    if payload.expected_delivery_date <= now {
        return Err(Error::InvalidInput {
            msg: "Expected delivery date must be in the future".to_string(),
        });
    }

    // Validate emergency contact
    if payload.emergency_contact.trim().is_empty() {
        return Err(Error::InvalidInput {
            msg: "Emergency contact is required".to_string(),
        });
    }

    Ok(())
}