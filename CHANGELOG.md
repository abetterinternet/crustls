# Changelog

## 0.6.0 (unreleased)

### Added

 - Add clone with OCSP for certified key (#85)
 - Make userdata a per-session config (#86). This makes it so callbacks
   can receive data associated with a specific TLS connection, whereas
   before they would receive data associated with a connection config
   (which might be shared across multiple connections).

### Changed

 - rustls_client_session_read_tls, rustls_client_session_write_tls,
   rustls_server_session_read_tls, and rustls_server_session_write_tls
   take a callback rather than copying bytes into a buffer. This can simplify
   user code significantly and in particular makes it harder for user code
   to accidentally drop bytes from the buffer. This introduces a new
   rustls_io_error type that is an alias for c_int. It wraps a value from
   `errno`. Both the updated read/write functions and the callbacks they
   receive return rustls_io_error.

## 0.5.0 - 2021-04-29

### Added

 - ALPN support for clients (#84)
 - Enumeration of ciphersuites (#79)

## 0.4.0 - 2021-03-18

### Added

 - Session storage (#64)
 - TLS version numbers (#65)

### Changed

 - Reading plaintext can now return RUSTLS_ALERT_CLOSE_NOTIFY. (#67)

### Removed

 - The rustls_cipher_signature_scheme name lookup. (#66)

## 0.3.0 - 2021-03-11

### Added

 - Expanded error handling: rustls_result has more variants. (#13)
 - Allow configuring custom trusted roots. (#16)
 - Use catch_unwind to prevent panicking across FFI. (#25)
 - Support for TLS servers. (#30)
 - Slice types: rustls_str, rustls_slice_bytes, rustls_slice_str,
   rustls_slice_slice_bytes, and rustls_slice_u16. (#54)
 - Callback for custom certificate verifier. (#51)
 - Callback for client hello inspection. (#50)

### Changed

 - By default, a rustls_client_config trusts no roots. (#13)

### Removed

 - Dependencies on `webpki-roots` and `env_logger`
 - Defensive zeroing when receiving write buffers from C. C code needs to
   ensure write buffers are initialized before handing to crustls. (#57)
