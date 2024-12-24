# Mama Pack - Maternal Health Tracking Canister

A decentralized maternal health tracking system built on the Internet Computer Protocol (ICP). This canister provides functionality for tracking maternal health records, monitoring high-risk cases, and managing prenatal appointments.

## Features

- Maternal profile management
- Health record tracking
- Risk monitoring
- Appointment management
- Pregnancy stage tracking
- Automated health status analysis

## Prerequisites

- [DFX SDK](https://internetcomputer.org/docs/current/developer-docs/setup/install) (v0.15.0 or later)
- [Rust](https://www.rust-lang.org/tools/install)
- [Node.js](https://nodejs.org/) (v18 or later)

## Quick Start

1. Clone the repository and navigate to the project:
```bash
git clone https://github.com/Stella-Achar-Oiro/mama-pack.git
cd mama-pack
```

2. Start the local replica:
```bash
dfx start --background
```

3. Deploy the canister:
```bash
dfx deploy
```

## Usage Examples

### 1. Create a Mother's Profile

```bash
dfx canister call mama-pack-backend create_mother_profile '(
  record {
    name = "Jane Doe";
    age = 28;
    blood_type = "O+";
    expected_delivery_date = 1751328000000000000;  # Future date in nanoseconds
    medical_history = vec { "No prior complications" };
    emergency_contact = "+1234567890";
  }
)'
```

### 2. Add a Health Record

```bash
dfx canister call mama-pack-backend add_health_record '(
  record {
    mother_id = 0;
    blood_pressure = "120/80";
    weight = 65.5;
    symptoms = vec { "mild nausea"; "fatigue" };
    notes = "Regular checkup";
    next_appointment = 1751328000000000000;
  }
)'
```

### 3. Query Health Records

```bash
# Get mother's profile
dfx canister call mama-pack-backend get_mother_profile '(0 : nat64)'

# Get health records
dfx canister call mama-pack-backend get_mother_health_records '(0 : nat64)'

# Get high-risk cases
dfx canister call mama-pack-backend get_high_risk_profiles

# Get upcoming appointments
dfx canister call mama-pack-backend get_upcoming_appointments '(7 : nat64)'
```

## API Reference

### Profile Management

- `create_mother_profile`: Create a new maternal health profile
- `get_mother_profile`: Retrieve a mother's profile by ID

### Health Records

- `add_health_record`: Add a new health record
- `get_mother_health_records`: Get all health records for a mother

### Risk Monitoring

- `get_critical_cases`: Get all mothers with critical health status
- `get_high_risk_profiles`: Get all high-risk profiles

### Appointment Management

- `get_upcoming_appointments`: Get upcoming appointments within specified days

## Data Types

### HealthStatus
```candid
type HealthStatus = variant {
    Normal;         // Regular health status
    NeedsAttention; // Minor concerns present
    Critical;       // Requires immediate attention
};
```

### PregnancyStage
```candid
type PregnancyStage = variant {
    FirstTrimester;   // Weeks 1-12
    SecondTrimester;  // Weeks 13-26
    ThirdTrimester;   // Weeks 27-40
    PostPartum;       // After delivery
};
```

## Testing with Candid UI

1. Deploy the canister locally
2. Open the Candid UI at: `http://localhost:4943/?canisterId=<canister_id>`
3. Test the functions in this order:
   - First: Create a profile using `create_mother_profile`
   - Then: Add health records using `add_health_record`
   - Finally: Query data using the various get methods

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## License

This project is licensed under the MIT License - see the LICENSE file for details.

## Security Considerations

- This is a prototype and should not be used in production without proper security audits
- Ensure proper access control mechanisms before deploying to mainnet
- Always validate and sanitize input data
- Consider encryption for sensitive medical information

## Support

For support, please open an issue in the GitHub repository or contact the development team.
