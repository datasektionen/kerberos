# Kerberos

Card reader CLI for SSO lookups.

## Usage

```bash
kerberos -h
```

### Key

Get a key from starting a new meeting in: [`karon.datasektionen.se`](https://karon.datasektionen.se/).

### Dependencies

#### Linux

- `libnfc`
- `pcsc-lite`
- `pcsc-tools`

Start the PC/SC daemon:

```bash
sudo systemctl start pcscd
```

#### Windows/macOS

No dependencies
