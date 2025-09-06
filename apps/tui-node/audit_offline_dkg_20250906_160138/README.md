# Offline DKG Security Audit Trail
Generated: Sat Sep  6 04:01:38 PM +08 2025
Test Type: 2-of-3 Threshold FROST DKG

## Directory Structure
- `mock_sd_card/`: Simulated SD card data exchanges
  - `coordinator/`: Data from coordinator node
  - `participant1/`: Data from participant 1
  - `participant2/`: Data from participant 2
- `logs/`: Execution logs from all nodes
- `artifacts/`: Generated keys and final outputs

## Security Checklist
- [ ] All data exchanges are encrypted
- [ ] No private keys exposed in logs
- [ ] Proper threshold verification (2-of-3)
- [ ] Deterministic key generation verified
- [ ] All participants properly authenticated
