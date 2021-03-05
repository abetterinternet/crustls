use crate::error::rustls_result;
use crate::error::rustls_result::NullParameter;
use crate::rslice::rustls_str;
use crate::{ffi_panic_boundary, ffi_panic_boundary_generic, ffi_panic_boundary_unit};
use libc::{c_char, c_ushort, c_void, size_t};
use rustls::{ProtocolVersion, SupportedCipherSuite};
use std::convert::TryInto;
use std::{cmp::min, slice};

#[repr(C)]
#[allow(dead_code)]
pub enum rustls_protocol_version {
    SSLv2 = 0x0200,
    SSLv3 = 0x0300,
    TLSv1_0 = 0x0301,
    TLSv1_1 = 0x0302,
    TLSv1_2 = 0x0303,
    TLSv1_3 = 0x0304,
}

pub(crate) fn rustls_protocol_version_from_u16(version_num: u16) -> Option<ProtocolVersion> {
    match version_num {
        0x0200 => Some(rustls::ProtocolVersion::SSLv2),
        0x0300 => Some(rustls::ProtocolVersion::SSLv3),
        0x0301 => Some(rustls::ProtocolVersion::TLSv1_0),
        0x0302 => Some(rustls::ProtocolVersion::TLSv1_1),
        0x0303 => Some(rustls::ProtocolVersion::TLSv1_2),
        0x0304 => Some(rustls::ProtocolVersion::TLSv1_3),
        _ => None,
    }
}

/// A snapshot of the registered TLS cipher suites as documented in
/// <https://www.iana.org/assignments/tls-parameters/tls-parameters.xhtml#tls-parameters-4>.
/// Future TLS protocol and/or rustls versions might support a super- or sub-set of these,
/// or both.
///
/// Meaning: this list is neither exhaustive, nor necessarily supported as ciphers may be added
/// or deprecated in the future. When neogiating a session with a remote endpoint, cipher
/// suites might be announced that are not present here.
///
#[repr(C)]
#[allow(dead_code)]
pub enum rustls_cipersuite {
    TLS_NULL_WITH_NULL_NULL = 0x0000,
    TLS_RSA_WITH_NULL_MD5 = 0x0001,
    TLS_RSA_WITH_NULL_SHA = 0x0002,
    TLS_RSA_EXPORT_WITH_RC4_40_MD5 = 0x0003,
    TLS_RSA_WITH_RC4_128_MD5 = 0x0004,
    TLS_RSA_WITH_RC4_128_SHA = 0x0005,
    TLS_RSA_EXPORT_WITH_RC2_CBC_40_MD5 = 0x0006,
    TLS_RSA_WITH_IDEA_CBC_SHA = 0x0007,
    TLS_RSA_EXPORT_WITH_DES40_CBC_SHA = 0x0008,
    TLS_RSA_WITH_DES_CBC_SHA = 0x0009,
    TLS_RSA_WITH_3DES_EDE_CBC_SHA = 0x000a,
    TLS_DH_DSS_EXPORT_WITH_DES40_CBC_SHA = 0x000b,
    TLS_DH_DSS_WITH_DES_CBC_SHA = 0x000c,
    TLS_DH_DSS_WITH_3DES_EDE_CBC_SHA = 0x000d,
    TLS_DH_RSA_EXPORT_WITH_DES40_CBC_SHA = 0x000e,
    TLS_DH_RSA_WITH_DES_CBC_SHA = 0x000f,
    TLS_DH_RSA_WITH_3DES_EDE_CBC_SHA = 0x0010,
    TLS_DHE_DSS_EXPORT_WITH_DES40_CBC_SHA = 0x0011,
    TLS_DHE_DSS_WITH_DES_CBC_SHA = 0x0012,
    TLS_DHE_DSS_WITH_3DES_EDE_CBC_SHA = 0x0013,
    TLS_DHE_RSA_EXPORT_WITH_DES40_CBC_SHA = 0x0014,
    TLS_DHE_RSA_WITH_DES_CBC_SHA = 0x0015,
    TLS_DHE_RSA_WITH_3DES_EDE_CBC_SHA = 0x0016,
    TLS_DH_anon_EXPORT_WITH_RC4_40_MD5 = 0x0017,
    TLS_DH_anon_WITH_RC4_128_MD5 = 0x0018,
    TLS_DH_anon_EXPORT_WITH_DES40_CBC_SHA = 0x0019,
    TLS_DH_anon_WITH_DES_CBC_SHA = 0x001a,
    TLS_DH_anon_WITH_3DES_EDE_CBC_SHA = 0x001b,
    SSL_FORTEZZA_KEA_WITH_NULL_SHA = 0x001c,
    SSL_FORTEZZA_KEA_WITH_FORTEZZA_CBC_SHA = 0x001d,
    TLS_KRB5_WITH_DES_CBC_SHA_or_SSL_FORTEZZA_KEA_WITH_RC4_128_SHA = 0x001e,
    TLS_KRB5_WITH_3DES_EDE_CBC_SHA = 0x001f,
    TLS_KRB5_WITH_RC4_128_SHA = 0x0020,
    TLS_KRB5_WITH_IDEA_CBC_SHA = 0x0021,
    TLS_KRB5_WITH_DES_CBC_MD5 = 0x0022,
    TLS_KRB5_WITH_3DES_EDE_CBC_MD5 = 0x0023,
    TLS_KRB5_WITH_RC4_128_MD5 = 0x0024,
    TLS_KRB5_WITH_IDEA_CBC_MD5 = 0x0025,
    TLS_KRB5_EXPORT_WITH_DES_CBC_40_SHA = 0x0026,
    TLS_KRB5_EXPORT_WITH_RC2_CBC_40_SHA = 0x0027,
    TLS_KRB5_EXPORT_WITH_RC4_40_SHA = 0x0028,
    TLS_KRB5_EXPORT_WITH_DES_CBC_40_MD5 = 0x0029,
    TLS_KRB5_EXPORT_WITH_RC2_CBC_40_MD5 = 0x002a,
    TLS_KRB5_EXPORT_WITH_RC4_40_MD5 = 0x002b,
    TLS_PSK_WITH_NULL_SHA = 0x002c,
    TLS_DHE_PSK_WITH_NULL_SHA = 0x002d,
    TLS_RSA_PSK_WITH_NULL_SHA = 0x002e,
    TLS_RSA_WITH_AES_128_CBC_SHA = 0x002f,
    TLS_DH_DSS_WITH_AES_128_CBC_SHA = 0x0030,
    TLS_DH_RSA_WITH_AES_128_CBC_SHA = 0x0031,
    TLS_DHE_DSS_WITH_AES_128_CBC_SHA = 0x0032,
    TLS_DHE_RSA_WITH_AES_128_CBC_SHA = 0x0033,
    TLS_DH_anon_WITH_AES_128_CBC_SHA = 0x0034,
    TLS_RSA_WITH_AES_256_CBC_SHA = 0x0035,
    TLS_DH_DSS_WITH_AES_256_CBC_SHA = 0x0036,
    TLS_DH_RSA_WITH_AES_256_CBC_SHA = 0x0037,
    TLS_DHE_DSS_WITH_AES_256_CBC_SHA = 0x0038,
    TLS_DHE_RSA_WITH_AES_256_CBC_SHA = 0x0039,
    TLS_DH_anon_WITH_AES_256_CBC_SHA = 0x003a,
    TLS_RSA_WITH_NULL_SHA256 = 0x003b,
    TLS_RSA_WITH_AES_128_CBC_SHA256 = 0x003c,
    TLS_RSA_WITH_AES_256_CBC_SHA256 = 0x003d,
    TLS_DH_DSS_WITH_AES_128_CBC_SHA256 = 0x003e,
    TLS_DH_RSA_WITH_AES_128_CBC_SHA256 = 0x003f,
    TLS_DHE_DSS_WITH_AES_128_CBC_SHA256 = 0x0040,
    TLS_RSA_WITH_CAMELLIA_128_CBC_SHA = 0x0041,
    TLS_DH_DSS_WITH_CAMELLIA_128_CBC_SHA = 0x0042,
    TLS_DH_RSA_WITH_CAMELLIA_128_CBC_SHA = 0x0043,
    TLS_DHE_DSS_WITH_CAMELLIA_128_CBC_SHA = 0x0044,
    TLS_DHE_RSA_WITH_CAMELLIA_128_CBC_SHA = 0x0045,
    TLS_DH_anon_WITH_CAMELLIA_128_CBC_SHA = 0x0046,
    TLS_ECDH_ECDSA_WITH_NULL_SHA_draft = 0x0047,
    TLS_ECDH_ECDSA_WITH_RC4_128_SHA_draft = 0x0048,
    TLS_ECDH_ECDSA_WITH_DES_CBC_SHA_draft = 0x0049,
    TLS_ECDH_ECDSA_WITH_3DES_EDE_CBC_SHA_draft = 0x004a,
    TLS_ECDH_ECDSA_WITH_AES_128_CBC_SHA_draft = 0x004b,
    TLS_ECDH_ECDSA_WITH_AES_256_CBC_SHA_draft = 0x004c,
    TLS_ECDH_ECNRA_WITH_DES_CBC_SHA_draft = 0x004d,
    TLS_ECDH_ECNRA_WITH_3DES_EDE_CBC_SHA_draft = 0x004e,
    TLS_ECMQV_ECDSA_NULL_SHA_draft = 0x004f,
    TLS_ECMQV_ECDSA_WITH_RC4_128_SHA_draft = 0x0050,
    TLS_ECMQV_ECDSA_WITH_DES_CBC_SHA_draft = 0x0051,
    TLS_ECMQV_ECDSA_WITH_3DES_EDE_CBC_SHA_draft = 0x0052,
    TLS_ECMQV_ECNRA_NULL_SHA_draft = 0x0053,
    TLS_ECMQV_ECNRA_WITH_RC4_128_SHA_draft = 0x0054,
    TLS_ECMQV_ECNRA_WITH_DES_CBC_SHA_draft = 0x0055,
    TLS_ECMQV_ECNRA_WITH_3DES_EDE_CBC_SHA_draft = 0x0056,
    TLS_ECDH_anon_NULL_WITH_SHA_draft = 0x0057,
    TLS_ECDH_anon_WITH_RC4_128_SHA_draft = 0x0058,
    TLS_ECDH_anon_WITH_DES_CBC_SHA_draft = 0x0059,
    TLS_ECDH_anon_WITH_3DES_EDE_CBC_SHA_draft = 0x005a,
    TLS_ECDH_anon_EXPORT_WITH_DES40_CBC_SHA_draft = 0x005b,
    TLS_ECDH_anon_EXPORT_WITH_RC4_40_SHA_draft = 0x005c,
    TLS_RSA_EXPORT1024_WITH_RC4_56_MD5 = 0x0060,
    TLS_RSA_EXPORT1024_WITH_RC2_CBC_56_MD5 = 0x0061,
    TLS_RSA_EXPORT1024_WITH_DES_CBC_SHA = 0x0062,
    TLS_DHE_DSS_EXPORT1024_WITH_DES_CBC_SHA = 0x0063,
    TLS_RSA_EXPORT1024_WITH_RC4_56_SHA = 0x0064,
    TLS_DHE_DSS_EXPORT1024_WITH_RC4_56_SHA = 0x0065,
    TLS_DHE_DSS_WITH_RC4_128_SHA = 0x0066,
    TLS_DHE_RSA_WITH_AES_128_CBC_SHA256 = 0x0067,
    TLS_DH_DSS_WITH_AES_256_CBC_SHA256 = 0x0068,
    TLS_DH_RSA_WITH_AES_256_CBC_SHA256 = 0x0069,
    TLS_DHE_DSS_WITH_AES_256_CBC_SHA256 = 0x006a,
    TLS_DHE_RSA_WITH_AES_256_CBC_SHA256 = 0x006b,
    TLS_DH_anon_WITH_AES_128_CBC_SHA256 = 0x006c,
    TLS_DH_anon_WITH_AES_256_CBC_SHA256 = 0x006d,
    TLS_DHE_DSS_WITH_3DES_EDE_CBC_RMD = 0x0072,
    TLS_DHE_DSS_WITH_AES_128_CBC_RMD = 0x0073,
    TLS_DHE_DSS_WITH_AES_256_CBC_RMD = 0x0074,
    TLS_DHE_RSA_WITH_3DES_EDE_CBC_RMD = 0x0077,
    TLS_DHE_RSA_WITH_AES_128_CBC_RMD = 0x0078,
    TLS_DHE_RSA_WITH_AES_256_CBC_RMD = 0x0079,
    TLS_RSA_WITH_3DES_EDE_CBC_RMD = 0x007c,
    TLS_RSA_WITH_AES_128_CBC_RMD = 0x007d,
    TLS_RSA_WITH_AES_256_CBC_RMD = 0x007e,
    TLS_GOSTR341094_WITH_28147_CNT_IMIT = 0x0080,
    TLS_GOSTR341001_WITH_28147_CNT_IMIT = 0x0081,
    TLS_GOSTR341094_WITH_NULL_GOSTR3411 = 0x0082,
    TLS_GOSTR341001_WITH_NULL_GOSTR3411 = 0x0083,
    TLS_RSA_WITH_CAMELLIA_256_CBC_SHA = 0x0084,
    TLS_DH_DSS_WITH_CAMELLIA_256_CBC_SHA = 0x0085,
    TLS_DH_RSA_WITH_CAMELLIA_256_CBC_SHA = 0x0086,
    TLS_DHE_DSS_WITH_CAMELLIA_256_CBC_SHA = 0x0087,
    TLS_DHE_RSA_WITH_CAMELLIA_256_CBC_SHA = 0x0088,
    TLS_DH_anon_WITH_CAMELLIA_256_CBC_SHA = 0x0089,
    TLS_PSK_WITH_RC4_128_SHA = 0x008a,
    TLS_PSK_WITH_3DES_EDE_CBC_SHA = 0x008b,
    TLS_PSK_WITH_AES_128_CBC_SHA = 0x008c,
    TLS_PSK_WITH_AES_256_CBC_SHA = 0x008d,
    TLS_DHE_PSK_WITH_RC4_128_SHA = 0x008e,
    TLS_DHE_PSK_WITH_3DES_EDE_CBC_SHA = 0x008f,
    TLS_DHE_PSK_WITH_AES_128_CBC_SHA = 0x0090,
    TLS_DHE_PSK_WITH_AES_256_CBC_SHA = 0x0091,
    TLS_RSA_PSK_WITH_RC4_128_SHA = 0x0092,
    TLS_RSA_PSK_WITH_3DES_EDE_CBC_SHA = 0x0093,
    TLS_RSA_PSK_WITH_AES_128_CBC_SHA = 0x0094,
    TLS_RSA_PSK_WITH_AES_256_CBC_SHA = 0x0095,
    TLS_RSA_WITH_SEED_CBC_SHA = 0x0096,
    TLS_DH_DSS_WITH_SEED_CBC_SHA = 0x0097,
    TLS_DH_RSA_WITH_SEED_CBC_SHA = 0x0098,
    TLS_DHE_DSS_WITH_SEED_CBC_SHA = 0x0099,
    TLS_DHE_RSA_WITH_SEED_CBC_SHA = 0x009a,
    TLS_DH_anon_WITH_SEED_CBC_SHA = 0x009b,
    TLS_RSA_WITH_AES_128_GCM_SHA256 = 0x009c,
    TLS_RSA_WITH_AES_256_GCM_SHA384 = 0x009d,
    TLS_DHE_RSA_WITH_AES_128_GCM_SHA256 = 0x009e,
    TLS_DHE_RSA_WITH_AES_256_GCM_SHA384 = 0x009f,
    TLS_DH_RSA_WITH_AES_128_GCM_SHA256 = 0x00a0,
    TLS_DH_RSA_WITH_AES_256_GCM_SHA384 = 0x00a1,
    TLS_DHE_DSS_WITH_AES_128_GCM_SHA256 = 0x00a2,
    TLS_DHE_DSS_WITH_AES_256_GCM_SHA384 = 0x00a3,
    TLS_DH_DSS_WITH_AES_128_GCM_SHA256 = 0x00a4,
    TLS_DH_DSS_WITH_AES_256_GCM_SHA384 = 0x00a5,
    TLS_DH_anon_WITH_AES_128_GCM_SHA256 = 0x00a6,
    TLS_DH_anon_WITH_AES_256_GCM_SHA384 = 0x00a7,
    TLS_PSK_WITH_AES_128_GCM_SHA256 = 0x00a8,
    TLS_PSK_WITH_AES_256_GCM_SHA384 = 0x00a9,
    TLS_DHE_PSK_WITH_AES_128_GCM_SHA256 = 0x00aa,
    TLS_DHE_PSK_WITH_AES_256_GCM_SHA384 = 0x00ab,
    TLS_RSA_PSK_WITH_AES_128_GCM_SHA256 = 0x00ac,
    TLS_RSA_PSK_WITH_AES_256_GCM_SHA384 = 0x00ad,
    TLS_PSK_WITH_AES_128_CBC_SHA256 = 0x00ae,
    TLS_PSK_WITH_AES_256_CBC_SHA384 = 0x00af,
    TLS_PSK_WITH_NULL_SHA256 = 0x00b0,
    TLS_PSK_WITH_NULL_SHA384 = 0x00b1,
    TLS_DHE_PSK_WITH_AES_128_CBC_SHA256 = 0x00b2,
    TLS_DHE_PSK_WITH_AES_256_CBC_SHA384 = 0x00b3,
    TLS_DHE_PSK_WITH_NULL_SHA256 = 0x00b4,
    TLS_DHE_PSK_WITH_NULL_SHA384 = 0x00b5,
    TLS_RSA_PSK_WITH_AES_128_CBC_SHA256 = 0x00b6,
    TLS_RSA_PSK_WITH_AES_256_CBC_SHA384 = 0x00b7,
    TLS_RSA_PSK_WITH_NULL_SHA256 = 0x00b8,
    TLS_RSA_PSK_WITH_NULL_SHA384 = 0x00b9,
    TLS_RSA_WITH_CAMELLIA_128_CBC_SHA256 = 0x00ba,
    TLS_DH_DSS_WITH_CAMELLIA_128_CBC_SHA256 = 0x00bb,
    TLS_DH_RSA_WITH_CAMELLIA_128_CBC_SHA256 = 0x00bc,
    TLS_DHE_DSS_WITH_CAMELLIA_128_CBC_SHA256 = 0x00bd,
    TLS_DHE_RSA_WITH_CAMELLIA_128_CBC_SHA256 = 0x00be,
    TLS_DH_anon_WITH_CAMELLIA_128_CBC_SHA256 = 0x00bf,
    TLS_RSA_WITH_CAMELLIA_256_CBC_SHA256 = 0x00c0,
    TLS_DH_DSS_WITH_CAMELLIA_256_CBC_SHA256 = 0x00c1,
    TLS_DH_RSA_WITH_CAMELLIA_256_CBC_SHA256 = 0x00c2,
    TLS_DHE_DSS_WITH_CAMELLIA_256_CBC_SHA256 = 0x00c3,
    TLS_DHE_RSA_WITH_CAMELLIA_256_CBC_SHA256 = 0x00c4,
    TLS_DH_anon_WITH_CAMELLIA_256_CBC_SHA256 = 0x00c5,
    TLS_EMPTY_RENEGOTIATION_INFO_SCSV = 0x00ff,
    TLS13_AES_128_GCM_SHA256 = 0x1301,
    TLS13_AES_256_GCM_SHA384 = 0x1302,
    TLS13_CHACHA20_POLY1305_SHA256 = 0x1303,
    TLS13_AES_128_CCM_SHA256 = 0x1304,
    TLS13_AES_128_CCM_8_SHA256 = 0x1305,
    TLS_ECDH_ECDSA_WITH_NULL_SHA = 0xc001,
    TLS_ECDH_ECDSA_WITH_RC4_128_SHA = 0xc002,
    TLS_ECDH_ECDSA_WITH_3DES_EDE_CBC_SHA = 0xc003,
    TLS_ECDH_ECDSA_WITH_AES_128_CBC_SHA = 0xc004,
    TLS_ECDH_ECDSA_WITH_AES_256_CBC_SHA = 0xc005,
    TLS_ECDHE_ECDSA_WITH_NULL_SHA = 0xc006,
    TLS_ECDHE_ECDSA_WITH_RC4_128_SHA = 0xc007,
    TLS_ECDHE_ECDSA_WITH_3DES_EDE_CBC_SHA = 0xc008,
    TLS_ECDHE_ECDSA_WITH_AES_128_CBC_SHA = 0xc009,
    TLS_ECDHE_ECDSA_WITH_AES_256_CBC_SHA = 0xc00a,
    TLS_ECDH_RSA_WITH_NULL_SHA = 0xc00b,
    TLS_ECDH_RSA_WITH_RC4_128_SHA = 0xc00c,
    TLS_ECDH_RSA_WITH_3DES_EDE_CBC_SHA = 0xc00d,
    TLS_ECDH_RSA_WITH_AES_128_CBC_SHA = 0xc00e,
    TLS_ECDH_RSA_WITH_AES_256_CBC_SHA = 0xc00f,
    TLS_ECDHE_RSA_WITH_NULL_SHA = 0xc010,
    TLS_ECDHE_RSA_WITH_RC4_128_SHA = 0xc011,
    TLS_ECDHE_RSA_WITH_3DES_EDE_CBC_SHA = 0xc012,
    TLS_ECDHE_RSA_WITH_AES_128_CBC_SHA = 0xc013,
    TLS_ECDHE_RSA_WITH_AES_256_CBC_SHA = 0xc014,
    TLS_ECDH_anon_WITH_NULL_SHA = 0xc015,
    TLS_ECDH_anon_WITH_RC4_128_SHA = 0xc016,
    TLS_ECDH_anon_WITH_3DES_EDE_CBC_SHA = 0xc017,
    TLS_ECDH_anon_WITH_AES_128_CBC_SHA = 0xc018,
    TLS_ECDH_anon_WITH_AES_256_CBC_SHA = 0xc019,
    TLS_SRP_SHA_WITH_3DES_EDE_CBC_SHA = 0xc01a,
    TLS_SRP_SHA_RSA_WITH_3DES_EDE_CBC_SHA = 0xc01b,
    TLS_SRP_SHA_DSS_WITH_3DES_EDE_CBC_SHA = 0xc01c,
    TLS_SRP_SHA_WITH_AES_128_CBC_SHA = 0xc01d,
    TLS_SRP_SHA_RSA_WITH_AES_128_CBC_SHA = 0xc01e,
    TLS_SRP_SHA_DSS_WITH_AES_128_CBC_SHA = 0xc01f,
    TLS_SRP_SHA_WITH_AES_256_CBC_SHA = 0xc020,
    TLS_SRP_SHA_RSA_WITH_AES_256_CBC_SHA = 0xc021,
    TLS_SRP_SHA_DSS_WITH_AES_256_CBC_SHA = 0xc022,
    TLS_ECDHE_ECDSA_WITH_AES_128_CBC_SHA256 = 0xc023,
    TLS_ECDHE_ECDSA_WITH_AES_256_CBC_SHA384 = 0xc024,
    TLS_ECDH_ECDSA_WITH_AES_128_CBC_SHA256 = 0xc025,
    TLS_ECDH_ECDSA_WITH_AES_256_CBC_SHA384 = 0xc026,
    TLS_ECDHE_RSA_WITH_AES_128_CBC_SHA256 = 0xc027,
    TLS_ECDHE_RSA_WITH_AES_256_CBC_SHA384 = 0xc028,
    TLS_ECDH_RSA_WITH_AES_128_CBC_SHA256 = 0xc029,
    TLS_ECDH_RSA_WITH_AES_256_CBC_SHA384 = 0xc02a,
    TLS_ECDHE_ECDSA_WITH_AES_128_GCM_SHA256 = 0xc02b,
    TLS_ECDHE_ECDSA_WITH_AES_256_GCM_SHA384 = 0xc02c,
    TLS_ECDH_ECDSA_WITH_AES_128_GCM_SHA256 = 0xc02d,
    TLS_ECDH_ECDSA_WITH_AES_256_GCM_SHA384 = 0xc02e,
    TLS_ECDHE_RSA_WITH_AES_128_GCM_SHA256 = 0xc02f,
    TLS_ECDHE_RSA_WITH_AES_256_GCM_SHA384 = 0xc030,
    TLS_ECDH_RSA_WITH_AES_128_GCM_SHA256 = 0xc031,
    TLS_ECDH_RSA_WITH_AES_256_GCM_SHA384 = 0xc032,
    TLS_ECDHE_PSK_WITH_RC4_128_SHA = 0xc033,
    TLS_ECDHE_PSK_WITH_3DES_EDE_CBC_SHA = 0xc034,
    TLS_ECDHE_PSK_WITH_AES_128_CBC_SHA = 0xc035,
    TLS_ECDHE_PSK_WITH_AES_256_CBC_SHA = 0xc036,
    TLS_ECDHE_PSK_WITH_AES_128_CBC_SHA256 = 0xc037,
    TLS_ECDHE_PSK_WITH_AES_256_CBC_SHA384 = 0xc038,
    TLS_ECDHE_PSK_WITH_NULL_SHA = 0xc039,
    TLS_ECDHE_PSK_WITH_NULL_SHA256 = 0xc03a,
    TLS_ECDHE_PSK_WITH_NULL_SHA384 = 0xc03b,
    TLS_RSA_WITH_ARIA_128_CBC_SHA256 = 0xc03c,
    TLS_RSA_WITH_ARIA_256_CBC_SHA384 = 0xc03d,
    TLS_DH_DSS_WITH_ARIA_128_CBC_SHA256 = 0xc03e,
    TLS_DH_DSS_WITH_ARIA_256_CBC_SHA384 = 0xc03f,
    TLS_DH_RSA_WITH_ARIA_128_CBC_SHA256 = 0xc040,
    TLS_DH_RSA_WITH_ARIA_256_CBC_SHA384 = 0xc041,
    TLS_DHE_DSS_WITH_ARIA_128_CBC_SHA256 = 0xc042,
    TLS_DHE_DSS_WITH_ARIA_256_CBC_SHA384 = 0xc043,
    TLS_DHE_RSA_WITH_ARIA_128_CBC_SHA256 = 0xc044,
    TLS_DHE_RSA_WITH_ARIA_256_CBC_SHA384 = 0xc045,
    TLS_DH_anon_WITH_ARIA_128_CBC_SHA256 = 0xc046,
    TLS_DH_anon_WITH_ARIA_256_CBC_SHA384 = 0xc047,
    TLS_ECDHE_ECDSA_WITH_ARIA_128_CBC_SHA256 = 0xc048,
    TLS_ECDHE_ECDSA_WITH_ARIA_256_CBC_SHA384 = 0xc049,
    TLS_ECDH_ECDSA_WITH_ARIA_128_CBC_SHA256 = 0xc04a,
    TLS_ECDH_ECDSA_WITH_ARIA_256_CBC_SHA384 = 0xc04b,
    TLS_ECDHE_RSA_WITH_ARIA_128_CBC_SHA256 = 0xc04c,
    TLS_ECDHE_RSA_WITH_ARIA_256_CBC_SHA384 = 0xc04d,
    TLS_ECDH_RSA_WITH_ARIA_128_CBC_SHA256 = 0xc04e,
    TLS_ECDH_RSA_WITH_ARIA_256_CBC_SHA384 = 0xc04f,
    TLS_RSA_WITH_ARIA_128_GCM_SHA256 = 0xc050,
    TLS_RSA_WITH_ARIA_256_GCM_SHA384 = 0xc051,
    TLS_DHE_RSA_WITH_ARIA_128_GCM_SHA256 = 0xc052,
    TLS_DHE_RSA_WITH_ARIA_256_GCM_SHA384 = 0xc053,
    TLS_DH_RSA_WITH_ARIA_128_GCM_SHA256 = 0xc054,
    TLS_DH_RSA_WITH_ARIA_256_GCM_SHA384 = 0xc055,
    TLS_DHE_DSS_WITH_ARIA_128_GCM_SHA256 = 0xc056,
    TLS_DHE_DSS_WITH_ARIA_256_GCM_SHA384 = 0xc057,
    TLS_DH_DSS_WITH_ARIA_128_GCM_SHA256 = 0xc058,
    TLS_DH_DSS_WITH_ARIA_256_GCM_SHA384 = 0xc059,
    TLS_DH_anon_WITH_ARIA_128_GCM_SHA256 = 0xc05a,
    TLS_DH_anon_WITH_ARIA_256_GCM_SHA384 = 0xc05b,
    TLS_ECDHE_ECDSA_WITH_ARIA_128_GCM_SHA256 = 0xc05c,
    TLS_ECDHE_ECDSA_WITH_ARIA_256_GCM_SHA384 = 0xc05d,
    TLS_ECDH_ECDSA_WITH_ARIA_128_GCM_SHA256 = 0xc05e,
    TLS_ECDH_ECDSA_WITH_ARIA_256_GCM_SHA384 = 0xc05f,
    TLS_ECDHE_RSA_WITH_ARIA_128_GCM_SHA256 = 0xc060,
    TLS_ECDHE_RSA_WITH_ARIA_256_GCM_SHA384 = 0xc061,
    TLS_ECDH_RSA_WITH_ARIA_128_GCM_SHA256 = 0xc062,
    TLS_ECDH_RSA_WITH_ARIA_256_GCM_SHA384 = 0xc063,
    TLS_PSK_WITH_ARIA_128_CBC_SHA256 = 0xc064,
    TLS_PSK_WITH_ARIA_256_CBC_SHA384 = 0xc065,
    TLS_DHE_PSK_WITH_ARIA_128_CBC_SHA256 = 0xc066,
    TLS_DHE_PSK_WITH_ARIA_256_CBC_SHA384 = 0xc067,
    TLS_RSA_PSK_WITH_ARIA_128_CBC_SHA256 = 0xc068,
    TLS_RSA_PSK_WITH_ARIA_256_CBC_SHA384 = 0xc069,
    TLS_PSK_WITH_ARIA_128_GCM_SHA256 = 0xc06a,
    TLS_PSK_WITH_ARIA_256_GCM_SHA384 = 0xc06b,
    TLS_DHE_PSK_WITH_ARIA_128_GCM_SHA256 = 0xc06c,
    TLS_DHE_PSK_WITH_ARIA_256_GCM_SHA384 = 0xc06d,
    TLS_RSA_PSK_WITH_ARIA_128_GCM_SHA256 = 0xc06e,
    TLS_RSA_PSK_WITH_ARIA_256_GCM_SHA384 = 0xc06f,
    TLS_ECDHE_PSK_WITH_ARIA_128_CBC_SHA256 = 0xc070,
    TLS_ECDHE_PSK_WITH_ARIA_256_CBC_SHA384 = 0xc071,
    TLS_ECDHE_ECDSA_WITH_CAMELLIA_128_CBC_SHA256 = 0xc072,
    TLS_ECDHE_ECDSA_WITH_CAMELLIA_256_CBC_SHA384 = 0xc073,
    TLS_ECDH_ECDSA_WITH_CAMELLIA_128_CBC_SHA256 = 0xc074,
    TLS_ECDH_ECDSA_WITH_CAMELLIA_256_CBC_SHA384 = 0xc075,
    TLS_ECDHE_RSA_WITH_CAMELLIA_128_CBC_SHA256 = 0xc076,
    TLS_ECDHE_RSA_WITH_CAMELLIA_256_CBC_SHA384 = 0xc077,
    TLS_ECDH_RSA_WITH_CAMELLIA_128_CBC_SHA256 = 0xc078,
    TLS_ECDH_RSA_WITH_CAMELLIA_256_CBC_SHA384 = 0xc079,
    TLS_RSA_WITH_CAMELLIA_128_GCM_SHA256 = 0xc07a,
    TLS_RSA_WITH_CAMELLIA_256_GCM_SHA384 = 0xc07b,
    TLS_DHE_RSA_WITH_CAMELLIA_128_GCM_SHA256 = 0xc07c,
    TLS_DHE_RSA_WITH_CAMELLIA_256_GCM_SHA384 = 0xc07d,
    TLS_DH_RSA_WITH_CAMELLIA_128_GCM_SHA256 = 0xc07e,
    TLS_DH_RSA_WITH_CAMELLIA_256_GCM_SHA384 = 0xc07f,
    TLS_DHE_DSS_WITH_CAMELLIA_128_GCM_SHA256 = 0xc080,
    TLS_DHE_DSS_WITH_CAMELLIA_256_GCM_SHA384 = 0xc081,
    TLS_DH_DSS_WITH_CAMELLIA_128_GCM_SHA256 = 0xc082,
    TLS_DH_DSS_WITH_CAMELLIA_256_GCM_SHA384 = 0xc083,
    TLS_DH_anon_WITH_CAMELLIA_128_GCM_SHA256 = 0xc084,
    TLS_DH_anon_WITH_CAMELLIA_256_GCM_SHA384 = 0xc085,
    TLS_ECDHE_ECDSA_WITH_CAMELLIA_128_GCM_SHA256 = 0xc086,
    TLS_ECDHE_ECDSA_WITH_CAMELLIA_256_GCM_SHA384 = 0xc087,
    TLS_ECDH_ECDSA_WITH_CAMELLIA_128_GCM_SHA256 = 0xc088,
    TLS_ECDH_ECDSA_WITH_CAMELLIA_256_GCM_SHA384 = 0xc089,
    TLS_ECDHE_RSA_WITH_CAMELLIA_128_GCM_SHA256 = 0xc08a,
    TLS_ECDHE_RSA_WITH_CAMELLIA_256_GCM_SHA384 = 0xc08b,
    TLS_ECDH_RSA_WITH_CAMELLIA_128_GCM_SHA256 = 0xc08c,
    TLS_ECDH_RSA_WITH_CAMELLIA_256_GCM_SHA384 = 0xc08d,
    TLS_PSK_WITH_CAMELLIA_128_GCM_SHA256 = 0xc08e,
    TLS_PSK_WITH_CAMELLIA_256_GCM_SHA384 = 0xc08f,
    TLS_DHE_PSK_WITH_CAMELLIA_128_GCM_SHA256 = 0xc090,
    TLS_DHE_PSK_WITH_CAMELLIA_256_GCM_SHA384 = 0xc091,
    TLS_RSA_PSK_WITH_CAMELLIA_128_GCM_SHA256 = 0xc092,
    TLS_RSA_PSK_WITH_CAMELLIA_256_GCM_SHA384 = 0xc093,
    TLS_PSK_WITH_CAMELLIA_128_CBC_SHA256 = 0xc094,
    TLS_PSK_WITH_CAMELLIA_256_CBC_SHA384 = 0xc095,
    TLS_DHE_PSK_WITH_CAMELLIA_128_CBC_SHA256 = 0xc096,
    TLS_DHE_PSK_WITH_CAMELLIA_256_CBC_SHA384 = 0xc097,
    TLS_RSA_PSK_WITH_CAMELLIA_128_CBC_SHA256 = 0xc098,
    TLS_RSA_PSK_WITH_CAMELLIA_256_CBC_SHA384 = 0xc099,
    TLS_ECDHE_PSK_WITH_CAMELLIA_128_CBC_SHA256 = 0xc09a,
    TLS_ECDHE_PSK_WITH_CAMELLIA_256_CBC_SHA384 = 0xc09b,
    TLS_RSA_WITH_AES_128_CCM = 0xc09c,
    TLS_RSA_WITH_AES_256_CCM = 0xc09d,
    TLS_DHE_RSA_WITH_AES_128_CCM = 0xc09e,
    TLS_DHE_RSA_WITH_AES_256_CCM = 0xc09f,
    TLS_RSA_WITH_AES_128_CCM_8 = 0xc0a0,
    TLS_RSA_WITH_AES_256_CCM_8 = 0xc0a1,
    TLS_DHE_RSA_WITH_AES_128_CCM_8 = 0xc0a2,
    TLS_DHE_RSA_WITH_AES_256_CCM_8 = 0xc0a3,
    TLS_PSK_WITH_AES_128_CCM = 0xc0a4,
    TLS_PSK_WITH_AES_256_CCM = 0xc0a5,
    TLS_DHE_PSK_WITH_AES_128_CCM = 0xc0a6,
    TLS_DHE_PSK_WITH_AES_256_CCM = 0xc0a7,
    TLS_PSK_WITH_AES_128_CCM_8 = 0xc0a8,
    TLS_PSK_WITH_AES_256_CCM_8 = 0xc0a9,
    TLS_PSK_DHE_WITH_AES_128_CCM_8 = 0xc0aa,
    TLS_PSK_DHE_WITH_AES_256_CCM_8 = 0xc0ab,
    TLS_ECDHE_RSA_WITH_CHACHA20_POLY1305_SHA256 = 0xcca8,
    TLS_ECDHE_ECDSA_WITH_CHACHA20_POLY1305_SHA256 = 0xcca9,
    TLS_DHE_RSA_WITH_CHACHA20_POLY1305_SHA256 = 0xccaa,
    TLS_PSK_WITH_CHACHA20_POLY1305_SHA256 = 0xccab,
    TLS_ECDHE_PSK_WITH_CHACHA20_POLY1305_SHA256 = 0xccac,
    TLS_DHE_PSK_WITH_CHACHA20_POLY1305_SHA256 = 0xccad,
    TLS_RSA_PSK_WITH_CHACHA20_POLY1305_SHA256 = 0xccae,
    SSL_RSA_FIPS_WITH_DES_CBC_SHA = 0xfefe,
    SSL_RSA_FIPS_WITH_3DES_EDE_CBC_SHA = 0xfeff,
}

pub(crate) fn rustls_supported_ciphersuite_from_u16(
    cipher_num: u16,
) -> Option<&'static rustls::SupportedCipherSuite> {
    for supported in rustls::ALL_CIPHERSUITES.iter() {
        if supported.suite.get_u16() == cipher_num {
            return Some(supported);
        }
    }
    None
}

/// Any context information the callback will receive when invoked.
#[allow(non_camel_case_types)]
pub type rustls_supported_ciphersuite_userdata = *mut c_void;

/// Prototype of a callback that receives numerical identifier and name of
/// a cipher suite supported by rustls.
/// `userdata` will be supplied as provided when registering the callback.
/// `id` gives the numerical identifier of the cipher suite.
/// 'name' gives the name of the suite as defined by rustls.
///
/// NOTE: the passed in `name` is only availabe during the callback invocation.
#[allow(non_camel_case_types)]
#[allow(dead_code)]
pub type rustls_supported_ciphersuite_callback = Option<
    unsafe extern "C" fn(
        userdata: rustls_supported_ciphersuite_userdata,
        id: c_ushort,
        name: &rustls_str,
    ),
>;

struct KnownCipherSuite<'a> {
    id: u16,
    name: &'a str,
}

static KNOWN_NAMES: &[KnownCipherSuite] = &[
    KnownCipherSuite {
        id: 0xc02b,
        name: "ECDHE-ECDSA-AES128-GCM-SHA256",
    },
    KnownCipherSuite {
        id: 0x1301,
        name: "TLS_AES_128_GCM_SHA256",
    },
    KnownCipherSuite {
        id: 0xcca8,
        name: "ECDHE-RSA-CHACHA20-POLY1305",
    },
    KnownCipherSuite {
        id: 0xc02c,
        name: "ECDHE-ECDSA-AES256-GCM-SHA384",
    },
    KnownCipherSuite {
        id: 0x1302,
        name: "TLS_AES_256_GCM_SHA384",
    },
    KnownCipherSuite {
        id: 0xc02f,
        name: "ECDHE-RSA-AES128-GCM-SHA256",
    },
    KnownCipherSuite {
        id: 0xc030,
        name: "ECDHE-RSA-AES256-GCM-SHA384",
    },
    KnownCipherSuite {
        id: 0xcca9,
        name: "ECDHE-ECDSA-CHACHA20-POLY1305",
    },
    KnownCipherSuite {
        id: 0x1303,
        name: "TLS_CHACHA20_POLY1305_SHA256",
    },
];

/// Get the 'standard' name for a supported cipher suite. See
/// <https://wiki.mozilla.org/Security/Server_Side_TLS> as an example
/// for definitions.
fn ciphersuite_get_name(cipher: &SupportedCipherSuite) -> String {
    for wellknown in KNOWN_NAMES.iter() {
        if wellknown.id == cipher.suite.get_u16() {
            return String::from(wellknown.name);
        }
    }
    String::from(format!("{:?}", cipher.suite))
}

// This is the same as a rustls_supported_ciphersuite_callback after unwrapping
// the Option (which is equivalent to checking for null).
#[allow(non_camel_case_types)]
type non_null_rustls_supported_ciphersuite_callback = unsafe extern "C" fn(
    userdata: rustls_supported_ciphersuite_userdata,
    id: c_ushort,
    name: &rustls_str,
);

#[no_mangle]
pub extern "C" fn rustls_supported_ciphersuite_iter(
    callback: rustls_supported_ciphersuite_callback,
    userdata: rustls_supported_ciphersuite_userdata,
) {
    ffi_panic_boundary_unit! {
        let callback: non_null_rustls_supported_ciphersuite_callback = match callback {
            Some(cb) => cb,
            None => return,
        };
        for cipher in rustls::ALL_CIPHERSUITES.iter() {
            let name = ciphersuite_get_name(cipher);
            let s: &str = &name;
            let rs: rustls_str = s.try_into().ok().unwrap();
            unsafe {
                callback(userdata, cipher.suite.get_u16(), &rs);
            }
        }
    }
}

/// Get the name of a CipherSuite, represented by the `suite` short value,
/// if known by the rustls library. For unknown schemes, this returns a string
/// with the scheme value in hex notation.
///
/// The caller provides `buf` for holding the string and gives its size as `len`
/// bytes. On return `out_n` carries the number of bytes copied into `buf`. The
/// `buf` is not NUL-terminated.
/// Returns `rustls_result::InsufficientSize` if the buffer was not large enough.
///
#[no_mangle]
pub extern "C" fn rustls_ciphersuite_get_name(
    suite: c_ushort,
    buf: *mut c_char,
    len: size_t,
    out_n: *mut size_t,
) -> rustls_result {
    ffi_panic_boundary! {
        let write_buf: &mut [u8] = unsafe {
            let out_n: &mut size_t = match out_n.as_mut() {
                Some(out_n) => out_n,
                None => return NullParameter,
            };
            *out_n = 0;
            if buf.is_null() {
                return NullParameter;
            }
            slice::from_raw_parts_mut(buf as *mut u8, len as usize)
        };
        let name = match rustls_supported_ciphersuite_from_u16(suite) {
            Some(s) => ciphersuite_get_name(s),
            None => format!("Unknown({:#06x})", suite)
        };
        let len: usize = min(write_buf.len() - 1, name.len());
        if len > write_buf.len() {
            return rustls_result::InsufficientSize;
        }
        write_buf[..len].copy_from_slice(&name.as_bytes()[..len]);
        unsafe {
            *out_n = len;
        }
        rustls_result::Ok
    }
}

/// All SignatureScheme currently defined in rustls.
/// At the moment not exposed by rustls itself.
#[no_mangle]
pub(crate) static ALL_SIGNATURE_SCHEMES: &[rustls::SignatureScheme] = &[
    rustls::SignatureScheme::RSA_PKCS1_SHA1,
    rustls::SignatureScheme::ECDSA_SHA1_Legacy,
    rustls::SignatureScheme::RSA_PKCS1_SHA256,
    rustls::SignatureScheme::ECDSA_NISTP256_SHA256,
    rustls::SignatureScheme::RSA_PKCS1_SHA384,
    rustls::SignatureScheme::ECDSA_NISTP384_SHA384,
    rustls::SignatureScheme::RSA_PKCS1_SHA512,
    rustls::SignatureScheme::ECDSA_NISTP521_SHA512,
    rustls::SignatureScheme::RSA_PSS_SHA256,
    rustls::SignatureScheme::RSA_PSS_SHA384,
    rustls::SignatureScheme::RSA_PSS_SHA512,
    rustls::SignatureScheme::ED25519,
    rustls::SignatureScheme::ED448,
];

/// Collect the u16 values of the given SignatureScheme slice, so they
/// can be exposed in our API.
pub(crate) fn rustls_signature_schemes_to_u16s(schemes: &[rustls::SignatureScheme]) -> Vec<u16> {
    let mut mapped_schemes: Vec<u16> = Vec::new();
    for s in schemes {
        mapped_schemes.push(s.get_u16());
    }
    mapped_schemes
}

/// Get the name of a SignatureScheme, represented by the `scheme` short value,
/// if known by the rustls library. For unknown schemes, this returns a string
/// with the scheme value in hex notation.
///
/// The caller provides `buf` for holding the string and gives its size as `len`
/// bytes. On return `out_n` carries the number of bytes copied into `buf`. The
/// `buf` is not NUL-terminated.
/// Returns `rustls_result::InsufficientSize` if the buffer was not large enough.
///
#[no_mangle]
pub extern "C" fn rustls_signature_scheme_get_name(
    scheme: c_ushort,
    buf: *mut c_char,
    len: size_t,
    out_n: *mut size_t,
) -> rustls_result {
    ffi_panic_boundary! {
        let write_buf: &mut [u8] = unsafe {
            let out_n: &mut size_t = match out_n.as_mut() {
                Some(out_n) => out_n,
                None => return NullParameter,
            };
            *out_n = 0;
            if buf.is_null() {
                return NullParameter;
            }
            slice::from_raw_parts_mut(buf as *mut u8, len as usize)
        };
        let get_name = || {
            for s in ALL_SIGNATURE_SCHEMES {
                if s.get_u16() == scheme {
                    return format!("{:?}", s);
                }
            }
            format!("Unknown({:#06x})", scheme)
        };
        let name = get_name();
        let len: usize = min(write_buf.len() - 1, name.len());
        if len > write_buf.len() {
            return rustls_result::InsufficientSize;
        }
        write_buf[..len].copy_from_slice(&name.as_bytes()[..len]);
        unsafe {
            *out_n = len;
        }
        rustls_result::Ok
    }
}
