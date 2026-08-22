# F-169, all, pass 1

**Reviewed**: working-tree diff, 15 files, 1,788 insertions and 11 deletions
**Verdict**: 4 defects, 0 smells, 0 nitpicks

## Defects

### D1, the bounded encrypted constructor ignores its allocation limit
`crates/oxml-opc/src/encryption.rs:352`

`decrypt_package` discards `PackageReadLimits`, then trusts the authenticated
stream's declared plaintext length for `Vec::with_capacity` at line 658. A
password-bearing but hostile package can therefore make the bounded constructor
reserve an attacker-selected amount before the normal ZIP limits run. This
contradicts the plan's bounded temporary-buffer requirement.

### D2, the parameter gate omits malformed salt and spin cases
`crates/oxml-opc/src/encryption.rs:735`

The named parameter test covers hash size, key size, block size, unknown hash,
and schema order, but never supplies a mismatched or oversized salt or an
excessive spin count. The approved test contract explicitly requires malformed
salts and spins to fail before allocation or decryption.

### D3, the tamper gate covers ciphertext but not HMAC material
`crates/oxml-opc/src/encryption.rs:1108`

The test changes only the final byte of `EncryptedPackage`. It does not modify
`encryptedHmacKey` or `encryptedHmacValue` in `EncryptionInfo`, although the
approved test contract requires modified ciphertext and HMAC bytes to fail
authentication before constructing a package.

### D4, the preservation gate does not exercise its bounded public path or full preservation contract
`crates/oxml-opc/src/encryption.rs:1127`

The test calls the internal decryptor and ordinary unbounded ZIP reader, then
checks only one binary part. Its fixture has no relationships or unmodelled XML,
and the test does not compare content types. It therefore does not prove the
approved assertions that the authenticated ZIP parses with limits and retains
relationships, content types, and preserved XML through deterministic save.

## Smells

None.

## Nitpicks

None.

## Not found

No additional correctness, contract, panic-safety, OOXML namespace or child-order,
test-gate, or structural-simplicity findings were found.
