use ::libc;
extern "C" {
    #[no_mangle]
    fn __ctype_b_loc() -> *mut *const libc::c_ushort;
    #[no_mangle]
    fn calloc(_: libc::c_ulong, _: libc::c_ulong) -> *mut libc::c_void;
    #[no_mangle]
    fn free(__ptr: *mut libc::c_void);
    #[no_mangle]
    fn bstr_add_mem_noex(b: *mut bstr, data: *const libc::c_void, len: size_t) -> *mut bstr;
    #[no_mangle]
    fn bstr_add_noex(bdestination: *mut bstr, bsource: *const bstr) -> *mut bstr;
    #[no_mangle]
    fn bstr_cmp_c_nocase(b: *const bstr, cstr: *const libc::c_char) -> libc::c_int;
    #[no_mangle]
    fn bstr_dup_mem(data: *const libc::c_void, len: size_t) -> *mut bstr;
    #[no_mangle]
    fn bstr_expand(b: *mut bstr, newsize: size_t) -> *mut bstr;
    #[no_mangle]
    fn bstr_free(b: *mut bstr);
    #[no_mangle]
    fn htp_table_add(
        table: *mut crate::src::htp_table::htp_table_t,
        key: *const bstr,
        element: *const libc::c_void,
    ) -> htp_status_t;
    #[no_mangle]
    fn htp_table_get(
        table: *const crate::src::htp_table::htp_table_t,
        key: *const bstr,
    ) -> *mut libc::c_void;
    #[no_mangle]
    fn htp_log(
        connp: *mut crate::src::htp_connection_parser::htp_connp_t,
        file: *const libc::c_char,
        line: libc::c_int,
        level: htp_log_level_t,
        code: libc::c_int,
        fmt: *const libc::c_char,
        _: ...
    );
    #[no_mangle]
    fn htp_parse_status(status: *mut bstr) -> libc::c_int;
    #[no_mangle]
    fn htp_is_space(c: libc::c_int) -> libc::c_int;
    #[no_mangle]
    fn htp_parse_protocol(protocol: *mut bstr) -> libc::c_int;
    #[no_mangle]
    fn htp_is_token(c: libc::c_int) -> libc::c_int;
    #[no_mangle]
    fn htp_is_lws(c: libc::c_int) -> libc::c_int;
    #[no_mangle]
    fn htp_chomp(data: *mut libc::c_uchar, len: *mut size_t) -> libc::c_int;
    #[no_mangle]
    fn htp_parse_content_length(
        b: *mut bstr,
        connp: *mut crate::src::htp_connection_parser::htp_connp_t,
    ) -> int64_t;
}
pub type __uint8_t = libc::c_uchar;
pub type __uint16_t = libc::c_ushort;
pub type __int32_t = libc::c_int;
pub type __int64_t = libc::c_long;
pub type __uint64_t = libc::c_ulong;
pub type __time_t = libc::c_long;
pub type __suseconds_t = libc::c_long;
pub type C2RustUnnamed = libc::c_uint;
pub const _ISalnum: C2RustUnnamed = 8;
pub const _ISpunct: C2RustUnnamed = 4;
pub const _IScntrl: C2RustUnnamed = 2;
pub const _ISblank: C2RustUnnamed = 1;
pub const _ISgraph: C2RustUnnamed = 32768;
pub const _ISprint: C2RustUnnamed = 16384;
pub const _ISspace: C2RustUnnamed = 8192;
pub const _ISxdigit: C2RustUnnamed = 4096;
pub const _ISdigit: C2RustUnnamed = 2048;
pub const _ISalpha: C2RustUnnamed = 1024;
pub const _ISlower: C2RustUnnamed = 512;
pub const _ISupper: C2RustUnnamed = 256;
pub type size_t = libc::c_ulong;
pub type int32_t = __int32_t;
pub type int64_t = __int64_t;
pub type uint8_t = __uint8_t;
pub type uint16_t = __uint16_t;
pub type uint64_t = __uint64_t;

pub type htp_status_t = libc::c_int;

/* *
 * Enumerates the ways in which servers respond to malformed data.
 */
pub type htp_unwanted_t = libc::c_uint;
/* * Responds with HTTP 404 status code. */
pub const HTP_UNWANTED_404: htp_unwanted_t = 404;
/* * Responds with HTTP 400 status code. */
pub const HTP_UNWANTED_400: htp_unwanted_t = 400;
/* * Ignores problem. */
pub const HTP_UNWANTED_IGNORE: htp_unwanted_t = 0;

/* *
 * Enumerates the possible approaches to handling invalid URL-encodings.
 */
pub type htp_url_encoding_handling_t = libc::c_uint;
/* * Decode invalid URL encodings. */
pub const HTP_URL_DECODE_PROCESS_INVALID: htp_url_encoding_handling_t = 2;
/* * Ignore invalid URL encodings, but remove the % from the data. */
pub const HTP_URL_DECODE_REMOVE_PERCENT: htp_url_encoding_handling_t = 1;
/* * Ignore invalid URL encodings and leave the % in the data. */
pub const HTP_URL_DECODE_PRESERVE_PERCENT: htp_url_encoding_handling_t = 0;

// A collection of unique parser IDs.
pub type htp_parser_id_t = libc::c_uint;
/* * multipart/form-data parser. */
pub const HTP_PARSER_MULTIPART: htp_parser_id_t = 1;
/* * application/x-www-form-urlencoded parser. */
pub const HTP_PARSER_URLENCODED: htp_parser_id_t = 0;
// Protocol version constants; an enum cannot be
// used here because we allow any properly-formatted protocol
// version (e.g., 1.3), even those that do not actually exist.
// A collection of possible data sources.
pub type htp_data_source_t = libc::c_uint;
/* * Transported in the request body. */
pub const HTP_SOURCE_BODY: htp_data_source_t = 3;
/* * Cookies. */
pub const HTP_SOURCE_COOKIE: htp_data_source_t = 2;
/* * Transported in the query string. */
pub const HTP_SOURCE_QUERY_STRING: htp_data_source_t = 1;
/* * Embedded in the URL. */
pub const HTP_SOURCE_URL: htp_data_source_t = 0;
pub type bstr = crate::src::bstr::bstr_t;

pub type htp_file_source_t = libc::c_uint;
pub const HTP_FILE_PUT: htp_file_source_t = 2;
pub const HTP_FILE_MULTIPART: htp_file_source_t = 1;

/* *
 * Possible states of a progressing transaction. Internally, progress will change
 * to the next state when the processing activities associated with that state
 * begin. For example, when we start to process request line bytes, the request
 * state will change from HTP_REQUEST_NOT_STARTED to HTP_REQUEST_LINE.*
 */
pub type htp_tx_res_progress_t = libc::c_uint;
pub const HTP_RESPONSE_COMPLETE: htp_tx_res_progress_t = 5;
pub const HTP_RESPONSE_TRAILER: htp_tx_res_progress_t = 4;
pub const HTP_RESPONSE_BODY: htp_tx_res_progress_t = 3;
pub const HTP_RESPONSE_HEADERS: htp_tx_res_progress_t = 2;
pub const HTP_RESPONSE_LINE: htp_tx_res_progress_t = 1;
pub const HTP_RESPONSE_NOT_STARTED: htp_tx_res_progress_t = 0;
pub type htp_tx_req_progress_t = libc::c_uint;
pub const HTP_REQUEST_COMPLETE: htp_tx_req_progress_t = 5;
pub const HTP_REQUEST_TRAILER: htp_tx_req_progress_t = 4;
pub const HTP_REQUEST_BODY: htp_tx_req_progress_t = 3;
pub const HTP_REQUEST_HEADERS: htp_tx_req_progress_t = 2;
pub const HTP_REQUEST_LINE: htp_tx_req_progress_t = 1;
pub const HTP_REQUEST_NOT_STARTED: htp_tx_req_progress_t = 0;
pub type htp_content_encoding_t = libc::c_uint;
pub const HTP_COMPRESSION_LZMA: htp_content_encoding_t = 4;
pub const HTP_COMPRESSION_DEFLATE: htp_content_encoding_t = 3;
pub const HTP_COMPRESSION_GZIP: htp_content_encoding_t = 2;
pub const HTP_COMPRESSION_NONE: htp_content_encoding_t = 1;
pub const HTP_COMPRESSION_UNKNOWN: htp_content_encoding_t = 0;
pub type htp_transfer_coding_t = libc::c_uint;
pub const HTP_CODING_INVALID: htp_transfer_coding_t = 4;
pub const HTP_CODING_CHUNKED: htp_transfer_coding_t = 3;
pub const HTP_CODING_IDENTITY: htp_transfer_coding_t = 2;
pub const HTP_CODING_NO_BODY: htp_transfer_coding_t = 1;
pub const HTP_CODING_UNKNOWN: htp_transfer_coding_t = 0;

pub type htp_table_alloc_t = libc::c_uint;
pub const HTP_TABLE_KEYS_REFERENCED: htp_table_alloc_t = 3;
pub const HTP_TABLE_KEYS_ADOPTED: htp_table_alloc_t = 2;
pub const HTP_TABLE_KEYS_COPIED: htp_table_alloc_t = 1;
pub const HTP_TABLE_KEYS_ALLOC_UKNOWN: htp_table_alloc_t = 0;
pub type htp_auth_type_t = libc::c_uint;
pub const HTP_AUTH_UNRECOGNIZED: htp_auth_type_t = 9;
pub const HTP_AUTH_DIGEST: htp_auth_type_t = 3;
pub const HTP_AUTH_BASIC: htp_auth_type_t = 2;
pub const HTP_AUTH_NONE: htp_auth_type_t = 1;
pub const HTP_AUTH_UNKNOWN: htp_auth_type_t = 0;

pub type htp_part_mode_t = libc::c_uint;
pub const MODE_DATA: htp_part_mode_t = 1;
pub const MODE_LINE: htp_part_mode_t = 0;

pub type htp_multipart_type_t = libc::c_uint;
pub const MULTIPART_PART_EPILOGUE: htp_multipart_type_t = 4;
pub const MULTIPART_PART_PREAMBLE: htp_multipart_type_t = 3;
pub const MULTIPART_PART_FILE: htp_multipart_type_t = 2;
pub const MULTIPART_PART_TEXT: htp_multipart_type_t = 1;
pub const MULTIPART_PART_UNKNOWN: htp_multipart_type_t = 0;
pub type htp_multipart_state_t = libc::c_uint;
pub const STATE_BOUNDARY_EAT_LWS_CR: htp_multipart_state_t = 6;
pub const STATE_BOUNDARY_EAT_LWS: htp_multipart_state_t = 5;
pub const STATE_BOUNDARY_IS_LAST2: htp_multipart_state_t = 4;
pub const STATE_BOUNDARY_IS_LAST1: htp_multipart_state_t = 3;
pub const STATE_BOUNDARY: htp_multipart_state_t = 2;
pub const STATE_DATA: htp_multipart_state_t = 1;
pub const STATE_INIT: htp_multipart_state_t = 0;

pub type htp_method_t = libc::c_uint;
pub const HTP_M_INVALID: htp_method_t = 28;
pub const HTP_M_MERGE: htp_method_t = 27;
pub const HTP_M_BASELINE_CONTROL: htp_method_t = 26;
pub const HTP_M_MKACTIVITY: htp_method_t = 25;
pub const HTP_M_MKWORKSPACE: htp_method_t = 24;
pub const HTP_M_REPORT: htp_method_t = 23;
pub const HTP_M_LABEL: htp_method_t = 22;
pub const HTP_M_UPDATE: htp_method_t = 21;
pub const HTP_M_CHECKIN: htp_method_t = 20;
pub const HTP_M_UNCHECKOUT: htp_method_t = 19;
pub const HTP_M_CHECKOUT: htp_method_t = 18;
pub const HTP_M_VERSION_CONTROL: htp_method_t = 17;
pub const HTP_M_UNLOCK: htp_method_t = 16;
pub const HTP_M_LOCK: htp_method_t = 15;
pub const HTP_M_MOVE: htp_method_t = 14;
pub const HTP_M_COPY: htp_method_t = 13;
pub const HTP_M_MKCOL: htp_method_t = 12;
pub const HTP_M_PROPPATCH: htp_method_t = 11;
pub const HTP_M_PROPFIND: htp_method_t = 10;
pub const HTP_M_PATCH: htp_method_t = 9;
pub const HTP_M_TRACE: htp_method_t = 8;
pub const HTP_M_OPTIONS: htp_method_t = 7;
pub const HTP_M_CONNECT: htp_method_t = 6;
pub const HTP_M_DELETE: htp_method_t = 5;
pub const HTP_M_POST: htp_method_t = 4;
pub const HTP_M_PUT: htp_method_t = 3;
pub const HTP_M_GET: htp_method_t = 2;
pub const HTP_M_HEAD: htp_method_t = 1;
pub const HTP_M_UNKNOWN: htp_method_t = 0;

pub type htp_time_t = crate::src::htp_connection_parser::timeval;
/* *
 * Enumerates all stream states. Each connection has two streams, one
 * inbound and one outbound. Their states are tracked separately.
 */
pub type htp_stream_state_t = libc::c_uint;
pub const HTP_STREAM_DATA: htp_stream_state_t = 9;
pub const HTP_STREAM_STOP: htp_stream_state_t = 6;
pub const HTP_STREAM_DATA_OTHER: htp_stream_state_t = 5;
pub const HTP_STREAM_TUNNEL: htp_stream_state_t = 4;
pub const HTP_STREAM_ERROR: htp_stream_state_t = 3;
pub const HTP_STREAM_CLOSED: htp_stream_state_t = 2;
pub const HTP_STREAM_OPEN: htp_stream_state_t = 1;
pub const HTP_STREAM_NEW: htp_stream_state_t = 0;

pub type htp_log_level_t = libc::c_uint;
pub const HTP_LOG_DEBUG2: htp_log_level_t = 6;
pub const HTP_LOG_DEBUG: htp_log_level_t = 5;
pub const HTP_LOG_INFO: htp_log_level_t = 4;
pub const HTP_LOG_NOTICE: htp_log_level_t = 3;
pub const HTP_LOG_WARNING: htp_log_level_t = 2;
pub const HTP_LOG_ERROR: htp_log_level_t = 1;
pub const HTP_LOG_NONE: htp_log_level_t = 0;
pub type htp_server_personality_t = libc::c_uint;
pub const HTP_SERVER_APACHE_2: htp_server_personality_t = 9;
pub const HTP_SERVER_IIS_7_5: htp_server_personality_t = 8;
pub const HTP_SERVER_IIS_7_0: htp_server_personality_t = 7;
pub const HTP_SERVER_IIS_6_0: htp_server_personality_t = 6;
pub const HTP_SERVER_IIS_5_1: htp_server_personality_t = 5;
pub const HTP_SERVER_IIS_5_0: htp_server_personality_t = 4;
pub const HTP_SERVER_IIS_4_0: htp_server_personality_t = 3;
pub const HTP_SERVER_IDS: htp_server_personality_t = 2;
pub const HTP_SERVER_GENERIC: htp_server_personality_t = 1;
pub const HTP_SERVER_MINIMAL: htp_server_personality_t = 0;

/* *
 * Generic response line parser.
 *
 * @param[in] connp
 * @return HTP status
 */
#[no_mangle]
pub unsafe extern "C" fn htp_parse_response_line_generic(
    mut connp: *mut crate::src::htp_connection_parser::htp_connp_t,
) -> htp_status_t {
    let mut tx: *mut crate::src::htp_transaction::htp_tx_t = (*connp).out_tx;
    let mut data: *mut libc::c_uchar = if (*(*tx).response_line).realptr.is_null() {
        ((*tx).response_line as *mut libc::c_uchar)
            .offset(::std::mem::size_of::<bstr>() as libc::c_ulong as isize)
    } else {
        (*(*tx).response_line).realptr
    };
    let mut len: size_t = (*(*tx).response_line).len;
    let mut pos: size_t = 0 as libc::c_int as size_t;
    (*tx).response_protocol = 0 as *mut bstr;
    (*tx).response_protocol_number = -(2 as libc::c_int);
    (*tx).response_status = 0 as *mut bstr;
    (*tx).response_status_number = -(1 as libc::c_int);
    (*tx).response_message = 0 as *mut bstr;
    // Ignore whitespace at the beginning of the line.
    while pos < len && htp_is_space(*data.offset(pos as isize) as libc::c_int) != 0 {
        pos = pos.wrapping_add(1)
    }
    let mut start: size_t = pos;
    // Find the end of the protocol string.
    while pos < len && htp_is_space(*data.offset(pos as isize) as libc::c_int) == 0 {
        pos = pos.wrapping_add(1)
    }
    if pos.wrapping_sub(start) == 0 as libc::c_int as libc::c_ulong {
        return 1 as libc::c_int;
    }
    (*tx).response_protocol = bstr_dup_mem(
        data.offset(start as isize) as *const libc::c_void,
        pos.wrapping_sub(start),
    );
    if (*tx).response_protocol.is_null() {
        return -(1 as libc::c_int);
    }
    (*tx).response_protocol_number = htp_parse_protocol((*tx).response_protocol);
    // Ignore whitespace after the response protocol.
    while pos < len && htp_is_space(*data.offset(pos as isize) as libc::c_int) != 0 {
        pos = pos.wrapping_add(1)
    }
    if pos == len {
        return 1 as libc::c_int;
    }
    start = pos;
    // Find the next whitespace character.
    while pos < len && htp_is_space(*data.offset(pos as isize) as libc::c_int) == 0 {
        pos = pos.wrapping_add(1)
    }
    if pos.wrapping_sub(start) == 0 as libc::c_int as libc::c_ulong {
        return 1 as libc::c_int;
    }
    (*tx).response_status = bstr_dup_mem(
        data.offset(start as isize) as *const libc::c_void,
        pos.wrapping_sub(start),
    );
    if (*tx).response_status.is_null() {
        return -(1 as libc::c_int);
    }
    (*tx).response_status_number = htp_parse_status((*tx).response_status);
    // Ignore whitespace that follows the status code.
    while pos < len
        && *(*__ctype_b_loc()).offset(*data.offset(pos as isize) as libc::c_int as isize)
            as libc::c_int
            & _ISspace as libc::c_int as libc::c_ushort as libc::c_int
            != 0
    {
        pos = pos.wrapping_add(1)
    }
    if pos == len {
        return 1 as libc::c_int;
    }
    // Assume the message stretches until the end of the line.
    (*tx).response_message = bstr_dup_mem(
        data.offset(pos as isize) as *const libc::c_void,
        len.wrapping_sub(pos),
    );
    if (*tx).response_message.is_null() {
        return -(1 as libc::c_int);
    }
    return 1 as libc::c_int;
}

/* *
 * Generic response header parser.
 *
 * @param[in] connp
 * @param[in] h
 * @param[in] data
 * @param[in] len
 * @return HTP status
 */
#[no_mangle]
pub unsafe extern "C" fn htp_parse_response_header_generic(
    mut connp: *mut crate::src::htp_connection_parser::htp_connp_t,
    mut h: *mut crate::src::htp_transaction::htp_header_t,
    mut data: *mut libc::c_uchar,
    mut len: size_t,
) -> htp_status_t {
    let mut name_start: size_t = 0;
    let mut name_end: size_t = 0;
    let mut value_start: size_t = 0;
    let mut value_end: size_t = 0;
    let mut prev: size_t = 0;
    htp_chomp(data, &mut len);
    name_start = 0 as libc::c_int as size_t;
    // Look for the first colon.
    let mut colon_pos: size_t = 0 as libc::c_int as size_t;
    while colon_pos < len && *data.offset(colon_pos as isize) as libc::c_int != ':' as i32 {
        colon_pos = colon_pos.wrapping_add(1)
    }
    if colon_pos == len {
        // Header line with a missing colon.
        (*h).flags = ((*h).flags as libc::c_ulonglong | 0x4 as libc::c_ulonglong) as uint64_t;
        (*h).flags = ((*h).flags as libc::c_ulonglong | 0x8 as libc::c_ulonglong) as uint64_t;
        if (*(*connp).out_tx).flags as libc::c_ulonglong & 0x4 as libc::c_ulonglong == 0 {
            // Only once per transaction.
            (*(*connp).out_tx).flags = ((*(*connp).out_tx).flags as libc::c_ulonglong
                | 0x4 as libc::c_ulonglong) as uint64_t;
            (*(*connp).out_tx).flags = ((*(*connp).out_tx).flags as libc::c_ulonglong
                | 0x8 as libc::c_ulonglong) as uint64_t;
            htp_log(
                connp,
                b"htp_response_generic.c\x00" as *const u8 as *const libc::c_char,
                147 as libc::c_int,
                HTP_LOG_WARNING,
                0 as libc::c_int,
                b"Response field invalid: missing colon.\x00" as *const u8 as *const libc::c_char,
            );
        }
        // Reset the position. We're going to treat this invalid header
        // as a header with an empty name. That will increase the probability
        // that the content will be inspected.
        colon_pos = 0 as libc::c_int as size_t;
        // suppress scan-build warning
        name_end = 0 as libc::c_int as size_t;
        value_start = 0 as libc::c_int as size_t
    } else {
        // Header line with a colon.
        if colon_pos == 0 as libc::c_int as libc::c_ulong {
            // Empty header name.
            (*h).flags = ((*h).flags as libc::c_ulonglong | 0x8 as libc::c_ulonglong) as uint64_t;
            if (*(*connp).out_tx).flags as libc::c_ulonglong & 0x8 as libc::c_ulonglong == 0 {
                // Only once per transaction.
                (*(*connp).out_tx).flags = ((*(*connp).out_tx).flags as libc::c_ulonglong
                    | 0x8 as libc::c_ulonglong)
                    as uint64_t;
                htp_log(
                    connp,
                    b"htp_response_generic.c\x00" as *const u8 as *const libc::c_char,
                    168 as libc::c_int,
                    HTP_LOG_WARNING,
                    0 as libc::c_int,
                    b"Response field invalid: empty name.\x00" as *const u8 as *const libc::c_char,
                );
            }
        }
        name_end = colon_pos;
        // Ignore unprintable after field-name.
        prev = name_end;
        while prev > name_start
            && *data.offset(prev.wrapping_sub(1 as libc::c_int as libc::c_ulong) as isize)
                as libc::c_int
                <= 0x20 as libc::c_int
        {
            prev = prev.wrapping_sub(1);
            name_end = name_end.wrapping_sub(1);
            (*h).flags = ((*h).flags as libc::c_ulonglong | 0x8 as libc::c_ulonglong) as uint64_t;
            if (*(*connp).out_tx).flags as libc::c_ulonglong & 0x8 as libc::c_ulonglong == 0 {
                // Only once per transaction.
                (*(*connp).out_tx).flags = ((*(*connp).out_tx).flags as libc::c_ulonglong
                    | 0x8 as libc::c_ulonglong)
                    as uint64_t;
                htp_log(
                    connp,
                    b"htp_response_generic.c\x00" as *const u8 as *const libc::c_char,
                    185 as libc::c_int,
                    HTP_LOG_WARNING,
                    0 as libc::c_int,
                    b"Response field invalid: LWS after name.\x00" as *const u8
                        as *const libc::c_char,
                );
            }
        }
        value_start = colon_pos.wrapping_add(1 as libc::c_int as libc::c_ulong)
    }
    // Header value.
    // Ignore LWS before field-content.
    while value_start < len && htp_is_lws(*data.offset(value_start as isize) as libc::c_int) != 0 {
        value_start = value_start.wrapping_add(1)
    }
    // Look for the end of field-content.
    value_end = len;
    // Check that the header name is a token.
    let mut i: size_t = name_start;
    while i < name_end {
        if htp_is_token(*data.offset(i as isize) as libc::c_int) == 0 {
            (*h).flags = ((*h).flags as libc::c_ulonglong | 0x8 as libc::c_ulonglong) as uint64_t;
            if (*(*connp).out_tx).flags as libc::c_ulonglong & 0x8 as libc::c_ulonglong == 0 {
                (*(*connp).out_tx).flags = ((*(*connp).out_tx).flags as libc::c_ulonglong
                    | 0x8 as libc::c_ulonglong)
                    as uint64_t;
                htp_log(
                    connp,
                    b"htp_response_generic.c\x00" as *const u8 as *const libc::c_char,
                    210 as libc::c_int,
                    HTP_LOG_WARNING,
                    0 as libc::c_int,
                    b"Response header name is not a token.\x00" as *const u8 as *const libc::c_char,
                );
            }
            break;
        } else {
            i = i.wrapping_add(1)
        }
    }
    i = value_start;
    while i < value_end {
        if *data.offset(i as isize) as libc::c_int == 0 as libc::c_int {
            htp_log(
                connp,
                b"htp_response_generic.c\x00" as *const u8 as *const libc::c_char,
                220 as libc::c_int,
                HTP_LOG_WARNING,
                0 as libc::c_int,
                b"Response header value contains null.\x00" as *const u8 as *const libc::c_char,
            );
            break;
        } else {
            i = i.wrapping_add(1)
        }
    }
    // Now extract the name and the value.
    (*h).name = bstr_dup_mem(
        data.offset(name_start as isize) as *const libc::c_void,
        name_end.wrapping_sub(name_start),
    );
    (*h).value = bstr_dup_mem(
        data.offset(value_start as isize) as *const libc::c_void,
        value_end.wrapping_sub(value_start),
    );
    if (*h).name.is_null() || (*h).value.is_null() {
        bstr_free((*h).name);
        bstr_free((*h).value);
        return -(1 as libc::c_int);
    }
    return 1 as libc::c_int;
}

/* *
 * Generic response header line(s) processor, which assembles folded lines
 * into a single buffer before invoking the parsing function.
 *
 * @param[in] connp
 * @param[in] data
 * @param[in] len
 * @return HTP status
 */
#[no_mangle]
pub unsafe extern "C" fn htp_process_response_header_generic(
    mut connp: *mut crate::src::htp_connection_parser::htp_connp_t,
    mut data: *mut libc::c_uchar,
    mut len: size_t,
) -> htp_status_t {
    // Create a new header structure.
    let mut h: *mut crate::src::htp_transaction::htp_header_t = calloc(
        1 as libc::c_int as libc::c_ulong,
        ::std::mem::size_of::<crate::src::htp_transaction::htp_header_t>() as libc::c_ulong,
    )
        as *mut crate::src::htp_transaction::htp_header_t;
    if h.is_null() {
        return -(1 as libc::c_int);
    }
    if htp_parse_response_header_generic(connp, h, data, len) != 1 as libc::c_int {
        free(h as *mut libc::c_void);
        return -(1 as libc::c_int);
    }
    // Do we already have a header with the same name?
    let mut h_existing: *mut crate::src::htp_transaction::htp_header_t =
        htp_table_get((*(*connp).out_tx).response_headers, (*h).name)
            as *mut crate::src::htp_transaction::htp_header_t;
    if !h_existing.is_null() {
        // Keep track of repeated same-name headers.
        if (*h_existing).flags as libc::c_ulonglong & 0x20 as libc::c_ulonglong
            == 0 as libc::c_int as libc::c_ulonglong
        {
            // This is the second occurence for this header.
            htp_log(
                connp,
                b"htp_response_generic.c\x00" as *const u8 as *const libc::c_char,
                267 as libc::c_int,
                HTP_LOG_WARNING,
                0 as libc::c_int,
                b"Repetition for header\x00" as *const u8 as *const libc::c_char,
            );
        } else if ((*(*connp).out_tx).res_header_repetitions as libc::c_int) < 64 as libc::c_int {
            (*(*connp).out_tx).res_header_repetitions =
                (*(*connp).out_tx).res_header_repetitions.wrapping_add(1)
        } else {
            bstr_free((*h).name);
            bstr_free((*h).value);
            free(h as *mut libc::c_void);
            return 1 as libc::c_int;
        }
        (*h_existing).flags =
            ((*h_existing).flags as libc::c_ulonglong | 0x20 as libc::c_ulonglong) as uint64_t;
        // For simplicity reasons, we count the repetitions of all headers
        // Having multiple C-L headers is against the RFC but many
        // browsers ignore the subsequent headers if the values are the same.
        if bstr_cmp_c_nocase(
            (*h).name,
            b"Content-Length\x00" as *const u8 as *const libc::c_char,
        ) == 0 as libc::c_int
        {
            // Don't use string comparison here because we want to
            // ignore small formatting differences.
            let mut existing_cl: int64_t = 0;
            let mut new_cl: int64_t = 0;
            existing_cl = htp_parse_content_length(
                (*h_existing).value,
                0 as *mut crate::src::htp_connection_parser::htp_connp_t,
            );
            new_cl = htp_parse_content_length(
                (*h).value,
                0 as *mut crate::src::htp_connection_parser::htp_connp_t,
            );
            if existing_cl == -(1 as libc::c_int) as libc::c_long
                || new_cl == -(1 as libc::c_int) as libc::c_long
                || existing_cl != new_cl
            {
                // Ambiguous response C-L value.
                htp_log(
                    connp,
                    b"htp_response_generic.c\x00" as *const u8 as *const libc::c_char,
                    293 as libc::c_int,
                    HTP_LOG_WARNING,
                    0 as libc::c_int,
                    b"Ambiguous response C-L value\x00" as *const u8 as *const libc::c_char,
                );
            }
        } else {
            // Add to the existing header.
            let mut new_value: *mut bstr = bstr_expand(
                (*h_existing).value,
                (*(*h_existing).value)
                    .len
                    .wrapping_add(2 as libc::c_int as libc::c_ulong)
                    .wrapping_add((*(*h).value).len),
            );
            if new_value.is_null() {
                bstr_free((*h).name);
                bstr_free((*h).value);
                free(h as *mut libc::c_void);
                return -(1 as libc::c_int);
            }
            (*h_existing).value = new_value;
            bstr_add_mem_noex(
                (*h_existing).value,
                b", \x00" as *const u8 as *const libc::c_char as *mut libc::c_uchar
                    as *const libc::c_void,
                2 as libc::c_int as size_t,
            );
            bstr_add_noex((*h_existing).value, (*h).value);
        }
        // The new header structure is no longer needed.
        bstr_free((*h).name);
        bstr_free((*h).value);
        free(h as *mut libc::c_void);
    } else if htp_table_add(
        (*(*connp).out_tx).response_headers,
        (*h).name,
        h as *const libc::c_void,
    ) != 1 as libc::c_int
    {
        bstr_free((*h).name);
        bstr_free((*h).value);
        free(h as *mut libc::c_void);
        return -(1 as libc::c_int);
    }
    return 1 as libc::c_int;
}
