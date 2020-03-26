use ::libc;
extern "C" {
    #[no_mangle]
    fn __ctype_b_loc() -> *mut *const libc::c_ushort;
    #[no_mangle]
    fn malloc(_: libc::c_ulong) -> *mut libc::c_void;
    #[no_mangle]
    fn realloc(_: *mut libc::c_void, _: libc::c_ulong) -> *mut libc::c_void;
    #[no_mangle]
    fn free(__ptr: *mut libc::c_void);
    #[no_mangle]
    fn memcpy(_: *mut libc::c_void, _: *const libc::c_void, _: libc::c_ulong) -> *mut libc::c_void;
    #[no_mangle]
    fn htp_list_array_get(
        l: *const crate::src::htp_list::htp_list_array_t,
        idx: size_t,
    ) -> *mut libc::c_void;
    #[no_mangle]
    fn bstr_add_mem(b: *mut bstr, data: *const libc::c_void, len: size_t) -> *mut bstr;
    #[no_mangle]
    fn bstr_adjust_len(b: *mut bstr, newlen: size_t);
    #[no_mangle]
    fn bstr_chr(b: *const bstr, c: libc::c_int) -> libc::c_int;
    #[no_mangle]
    fn bstr_cmp_c_nocase(b: *const bstr, cstr: *const libc::c_char) -> libc::c_int;
    #[no_mangle]
    fn bstr_dup_c(cstr: *const libc::c_char) -> *mut bstr;
    #[no_mangle]
    fn bstr_dup_lower(b: *const bstr) -> *mut bstr;
    #[no_mangle]
    fn bstr_dup_mem(data: *const libc::c_void, len: size_t) -> *mut bstr;
    #[no_mangle]
    fn bstr_free(b: *mut bstr);
    #[no_mangle]
    fn bstr_index_of_c_nocase(bhaystack: *const bstr, cneedle: *const libc::c_char) -> libc::c_int;
    #[no_mangle]
    fn bstr_index_of_c_nocasenorzero(
        bhaystack: *const bstr,
        cneedle: *const libc::c_char,
    ) -> libc::c_int;
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
    fn htp_hook_run_all(
        hook: *mut crate::src::htp_hooks::htp_hook_t,
        user_data: *mut libc::c_void,
    ) -> htp_status_t;
    #[no_mangle]
    fn htp_is_folding_char(c: libc::c_int) -> libc::c_int;
    #[no_mangle]
    fn htp_connp_is_line_folded(data: *mut libc::c_uchar, len: size_t) -> libc::c_int;
    #[no_mangle]
    fn htp_chomp(data: *mut libc::c_uchar, len: *mut size_t) -> libc::c_int;
    #[no_mangle]
    fn htp_tx_state_response_complete_ex(
        tx: *mut crate::src::htp_transaction::htp_tx_t,
        hybrid_mode: libc::c_int,
    ) -> htp_status_t;
    #[no_mangle]
    fn htp_tx_res_process_body_data_ex(
        tx: *mut crate::src::htp_transaction::htp_tx_t,
        data: *const libc::c_void,
        len: size_t,
    ) -> htp_status_t;
    #[no_mangle]
    fn htp_treat_response_line_as_body(data: *const uint8_t, len: size_t) -> libc::c_int;
    #[no_mangle]
    fn htp_tx_state_response_headers(
        tx: *mut crate::src::htp_transaction::htp_tx_t,
    ) -> htp_status_t;
    #[no_mangle]
    fn htp_table_get_c(
        table: *const crate::src::htp_table::htp_table_t,
        ckey: *const libc::c_char,
    ) -> *mut libc::c_void;
    #[no_mangle]
    fn htp_parse_content_length(
        b: *mut bstr,
        connp: *mut crate::src::htp_connection_parser::htp_connp_t,
    ) -> int64_t;
    #[no_mangle]
    fn htp_parse_chunked_length(data: *mut libc::c_uchar, len: size_t) -> int64_t;
    #[no_mangle]
    fn htp_is_space(c: libc::c_int) -> libc::c_int;
    #[no_mangle]
    fn htp_tx_state_response_line(tx: *mut crate::src::htp_transaction::htp_tx_t) -> htp_status_t;
    #[no_mangle]
    fn htp_connp_is_line_ignorable(
        connp: *mut crate::src::htp_connection_parser::htp_connp_t,
        data: *mut libc::c_uchar,
        len: size_t,
    ) -> libc::c_int;
    #[no_mangle]
    fn htp_table_clear(table: *mut crate::src::htp_table::htp_table_t);
    #[no_mangle]
    fn htp_table_get_index(
        table: *const crate::src::htp_table::htp_table_t,
        idx: size_t,
        key: *mut *mut bstr,
    ) -> *mut libc::c_void;
    #[no_mangle]
    fn htp_table_size(table: *const crate::src::htp_table::htp_table_t) -> size_t;
    #[no_mangle]
    fn htp_connp_is_line_terminator(
        connp: *mut crate::src::htp_connection_parser::htp_connp_t,
        data: *mut libc::c_uchar,
        len: size_t,
    ) -> libc::c_int;
    #[no_mangle]
    fn htp_conn_track_outbound_data(
        conn: *mut crate::src::htp_connection::htp_conn_t,
        len: size_t,
        timestamp: *const htp_time_t,
    );
    #[no_mangle]
    fn htp_tx_state_response_start(tx: *mut crate::src::htp_transaction::htp_tx_t) -> htp_status_t;
    #[no_mangle]
    fn htp_connp_REQ_FINALIZE(
        connp: *mut crate::src::htp_connection_parser::htp_connp_t,
    ) -> htp_status_t;
    #[no_mangle]
    fn htp_uri_alloc() -> *mut crate::src::htp_util::htp_uri_t;
    #[no_mangle]
    fn htp_connp_tx_create(
        connp: *mut crate::src::htp_connection_parser::htp_connp_t,
    ) -> *mut crate::src::htp_transaction::htp_tx_t;
    #[no_mangle]
    fn htp_tx_state_request_complete(
        tx: *mut crate::src::htp_transaction::htp_tx_t,
    ) -> htp_status_t;
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
 * Sends outstanding connection data to the currently active data receiver hook.
 *
 * @param[in] connp
 * @param[in] is_last
 * @return HTP_OK, or a value returned from a callback.
 */
unsafe extern "C" fn htp_connp_res_receiver_send_data(
    mut connp: *mut crate::src::htp_connection_parser::htp_connp_t,
    mut is_last: libc::c_int,
) -> htp_status_t {
    if (*connp).out_data_receiver_hook.is_null() {
        return 1 as libc::c_int;
    }
    let mut d: crate::src::htp_transaction::htp_tx_data_t =
        crate::src::htp_transaction::htp_tx_data_t {
            tx: 0 as *mut crate::src::htp_transaction::htp_tx_t,
            data: 0 as *const libc::c_uchar,
            len: 0,
            is_last: 0,
        };
    d.tx = (*connp).out_tx;
    d.data = (*connp)
        .out_current_data
        .offset((*connp).out_current_receiver_offset as isize);
    d.len = ((*connp).out_current_read_offset - (*connp).out_current_receiver_offset) as size_t;
    d.is_last = is_last;
    let mut rc: htp_status_t = htp_hook_run_all(
        (*connp).out_data_receiver_hook,
        &mut d as *mut crate::src::htp_transaction::htp_tx_data_t as *mut libc::c_void,
    );
    if rc != 1 as libc::c_int {
        return rc;
    }
    (*connp).out_current_receiver_offset = (*connp).out_current_read_offset;
    return 1 as libc::c_int;
}

/* *
 * Finalizes an existing data receiver hook by sending any outstanding data to it. The
 * hook is then removed so that it receives no more data.
 *
 * @param[in] connp
 * @return HTP_OK, or a value returned from a callback.
 */
#[no_mangle]
pub unsafe extern "C" fn htp_connp_res_receiver_finalize_clear(
    mut connp: *mut crate::src::htp_connection_parser::htp_connp_t,
) -> htp_status_t {
    if (*connp).out_data_receiver_hook.is_null() {
        return 1 as libc::c_int;
    }
    let mut rc: htp_status_t = htp_connp_res_receiver_send_data(connp, 1 as libc::c_int);
    (*connp).out_data_receiver_hook = 0 as *mut crate::src::htp_hooks::htp_hook_t;
    return rc;
}

/* *
 * Configures the data receiver hook. If there is a previous hook, it will be finalized and cleared.
 *
 * @param[in] connp
 * @param[in] data_receiver_hook
 * @return HTP_OK, or a value returned from a callback.
 */
unsafe extern "C" fn htp_connp_res_receiver_set(
    mut connp: *mut crate::src::htp_connection_parser::htp_connp_t,
    mut data_receiver_hook: *mut crate::src::htp_hooks::htp_hook_t,
) -> htp_status_t {
    htp_connp_res_receiver_finalize_clear(connp);
    (*connp).out_data_receiver_hook = data_receiver_hook;
    (*connp).out_current_receiver_offset = (*connp).out_current_read_offset;
    return 1 as libc::c_int;
}

/* *
 * Handles request parser state changes. At the moment, this function is used only
 * to configure data receivers, which are sent raw connection data.
 *
 * @param[in] connp
 * @return HTP_OK, or a value returned from a callback.
 */
unsafe extern "C" fn htp_res_handle_state_change(
    mut connp: *mut crate::src::htp_connection_parser::htp_connp_t,
) -> htp_status_t {
    if (*connp).out_state_previous == (*connp).out_state {
        return 1 as libc::c_int;
    }
    if (*connp).out_state
        == Some(
            htp_connp_RES_HEADERS
                as unsafe extern "C" fn(
                    _: *mut crate::src::htp_connection_parser::htp_connp_t,
                ) -> htp_status_t,
        )
    {
        let mut rc: htp_status_t = 1 as libc::c_int;
        match (*(*connp).out_tx).response_progress as libc::c_uint {
            2 => {
                rc = htp_connp_res_receiver_set(
                    connp,
                    (*(*(*connp).out_tx).cfg).hook_response_header_data,
                )
            }
            4 => {
                rc = htp_connp_res_receiver_set(
                    connp,
                    (*(*(*connp).out_tx).cfg).hook_response_trailer_data,
                )
            }
            _ => {}
        }
        if rc != 1 as libc::c_int {
            return rc;
        }
    }
    // Same comment as in htp_req_handle_state_change(). Below is a copy.
    // Initially, I had the finalization of raw data sending here, but that
    // caused the last REQUEST_HEADER_DATA hook to be invoked after the
    // REQUEST_HEADERS hook -- which I thought made no sense. For that reason,
    // the finalization is now initiated from the request header processing code,
    // which is less elegant but provides a better user experience. Having some
    // (or all) hooks to be invoked on state change might work better.
    (*connp).out_state_previous = (*connp).out_state;
    return 1 as libc::c_int;
}

/* *
 * If there is any data left in the outbound data chunk, this function will preserve
 * it for later consumption. The maximum amount accepted for buffering is controlled
 * by htp_config_t::field_limit_hard.
 *
 * @param[in] connp
 * @return HTP_OK, or HTP_ERROR on fatal failure.
 */
unsafe extern "C" fn htp_connp_res_buffer(
    mut connp: *mut crate::src::htp_connection_parser::htp_connp_t,
) -> htp_status_t {
    if (*connp).out_current_data.is_null() {
        return 1 as libc::c_int;
    }
    let mut data: *mut libc::c_uchar = (*connp)
        .out_current_data
        .offset((*connp).out_current_consume_offset as isize);
    let mut len: size_t =
        ((*connp).out_current_read_offset - (*connp).out_current_consume_offset) as size_t;
    // Check the hard (buffering) limit.
    let mut newlen: size_t = (*connp).out_buf_size.wrapping_add(len);
    // When calculating the size of the buffer, take into account the
    // space we're using for the response header buffer.
    if !(*connp).out_header.is_null() {
        newlen =
            (newlen as libc::c_ulong).wrapping_add((*(*connp).out_header).len) as size_t as size_t
    }
    if newlen > (*(*(*connp).out_tx).cfg).field_limit_hard {
        htp_log(
            connp,
            b"htp_response.c\x00" as *const u8 as *const libc::c_char,
            212 as libc::c_int,
            HTP_LOG_ERROR,
            0 as libc::c_int,
            b"Response the buffer limit: size %zd limit %zd.\x00" as *const u8
                as *const libc::c_char,
            newlen,
            (*(*(*connp).out_tx).cfg).field_limit_hard,
        );
        return -(1 as libc::c_int);
    }
    // Copy the data remaining in the buffer.
    if (*connp).out_buf.is_null() {
        (*connp).out_buf = malloc(len) as *mut libc::c_uchar;
        if (*connp).out_buf.is_null() {
            return -(1 as libc::c_int);
        }
        memcpy(
            (*connp).out_buf as *mut libc::c_void,
            data as *const libc::c_void,
            len,
        );
        (*connp).out_buf_size = len
    } else {
        let mut newsize: size_t = (*connp).out_buf_size.wrapping_add(len);
        let mut newbuf: *mut libc::c_uchar =
            realloc((*connp).out_buf as *mut libc::c_void, newsize) as *mut libc::c_uchar;
        if newbuf.is_null() {
            return -(1 as libc::c_int);
        }
        (*connp).out_buf = newbuf;
        memcpy(
            (*connp).out_buf.offset((*connp).out_buf_size as isize) as *mut libc::c_void,
            data as *const libc::c_void,
            len,
        );
        (*connp).out_buf_size = newsize
    }
    // Reset the consumer position.
    (*connp).out_current_consume_offset = (*connp).out_current_read_offset;
    return 1 as libc::c_int;
}

/* *
 * Returns to the caller the memory region that should be processed next. This function
 * hides away the buffering process from the rest of the code, allowing it to work with
 * non-buffered data that's in the outbound chunk, or buffered data that's in our structures.
 *
 * @param[in] connp
 * @param[out] data
 * @param[out] len
 * @return HTP_OK
 */
unsafe extern "C" fn htp_connp_res_consolidate_data(
    mut connp: *mut crate::src::htp_connection_parser::htp_connp_t,
    mut data: *mut *mut libc::c_uchar,
    mut len: *mut size_t,
) -> htp_status_t {
    if (*connp).out_buf.is_null() {
        // We do not have any data buffered; point to the current data chunk.
        *data = (*connp)
            .out_current_data
            .offset((*connp).out_current_consume_offset as isize);
        *len = ((*connp).out_current_read_offset - (*connp).out_current_consume_offset) as size_t
    } else {
        // We do have data in the buffer. Add data from the current
        // chunk, and point to the consolidated buffer.
        if htp_connp_res_buffer(connp) != 1 as libc::c_int {
            return -(1 as libc::c_int);
        }
        *data = (*connp).out_buf;
        *len = (*connp).out_buf_size
    }
    return 1 as libc::c_int;
}

/* *
 * Clears buffered outbound data and resets the consumer position to the reader position.
 *
 * @param[in] connp
 */
unsafe extern "C" fn htp_connp_res_clear_buffer(
    mut connp: *mut crate::src::htp_connection_parser::htp_connp_t,
) {
    (*connp).out_current_consume_offset = (*connp).out_current_read_offset;
    if !(*connp).out_buf.is_null() {
        free((*connp).out_buf as *mut libc::c_void);
        (*connp).out_buf = 0 as *mut libc::c_uchar;
        (*connp).out_buf_size = 0 as libc::c_int as size_t
    };
}

/* *
 * Consumes bytes until the end of the current line.
 *
 * @param[in] connp
 * @returns HTP_OK on state change, HTP_ERROR on error, or HTP_DATA when more data is needed.
 */
#[no_mangle]
pub unsafe extern "C" fn htp_connp_RES_BODY_CHUNKED_DATA_END(
    mut connp: *mut crate::src::htp_connection_parser::htp_connp_t,
) -> htp_status_t {
    loop
    // TODO We shouldn't really see anything apart from CR and LF,
    //      so we should warn about anything else.
    {
        if (*connp).out_current_read_offset < (*connp).out_current_len {
            (*connp).out_next_byte = *(*connp)
                .out_current_data
                .offset((*connp).out_current_read_offset as isize)
                as libc::c_int;
            (*connp).out_current_read_offset += 1;
            (*connp).out_current_consume_offset += 1;
            (*connp).out_stream_offset += 1
        } else {
            return 2 as libc::c_int;
        }
        (*(*connp).out_tx).response_message_len += 1;
        if (*connp).out_next_byte == '\n' as i32 {
            (*connp).out_state = Some(
                htp_connp_RES_BODY_CHUNKED_LENGTH
                    as unsafe extern "C" fn(
                        _: *mut crate::src::htp_connection_parser::htp_connp_t,
                    ) -> htp_status_t,
            );
            return 1 as libc::c_int;
        }
    }
}

/* *
 * Processes a chunk of data.
 *
 * @param[in] connp
 * @returns HTP_OK on state change, HTP_ERROR on error, or HTP_DATA when more data is needed.
 */
#[no_mangle]
pub unsafe extern "C" fn htp_connp_RES_BODY_CHUNKED_DATA(
    mut connp: *mut crate::src::htp_connection_parser::htp_connp_t,
) -> htp_status_t {
    let mut bytes_to_consume: size_t = 0;
    // Determine how many bytes we can consume.
    if (*connp).out_current_len - (*connp).out_current_read_offset >= (*connp).out_chunked_length {
        bytes_to_consume = (*connp).out_chunked_length as size_t
    } else {
        bytes_to_consume = ((*connp).out_current_len - (*connp).out_current_read_offset) as size_t
    }
    if bytes_to_consume == 0 as libc::c_int as libc::c_ulong {
        return 2 as libc::c_int;
    }
    // Consume the data.
    let mut rc: htp_status_t = htp_tx_res_process_body_data_ex(
        (*connp).out_tx,
        (*connp)
            .out_current_data
            .offset((*connp).out_current_read_offset as isize) as *const libc::c_void,
        bytes_to_consume,
    );
    if rc != 1 as libc::c_int {
        return rc;
    }
    // Adjust the counters.
    (*connp).out_current_read_offset = ((*connp).out_current_read_offset as libc::c_ulong)
        .wrapping_add(bytes_to_consume) as int64_t
        as int64_t;
    (*connp).out_current_consume_offset = ((*connp).out_current_consume_offset as libc::c_ulong)
        .wrapping_add(bytes_to_consume) as int64_t
        as int64_t;
    (*connp).out_stream_offset = ((*connp).out_stream_offset as libc::c_ulong)
        .wrapping_add(bytes_to_consume) as int64_t as int64_t;
    (*connp).out_chunked_length = ((*connp).out_chunked_length as libc::c_ulong)
        .wrapping_sub(bytes_to_consume) as int64_t as int64_t;
    // Have we seen the entire chunk?
    if (*connp).out_chunked_length == 0 as libc::c_int as libc::c_long {
        (*connp).out_state = Some(
            htp_connp_RES_BODY_CHUNKED_DATA_END
                as unsafe extern "C" fn(
                    _: *mut crate::src::htp_connection_parser::htp_connp_t,
                ) -> htp_status_t,
        );
        return 1 as libc::c_int;
    }
    return 2 as libc::c_int;
}

/* *
 * Peeks ahead into the data to try to see if it starts with a valid Chunked
 * length field.
 *
 * @returns 1 if it looks valid, 0 if it looks invalid
 */
#[inline]
unsafe extern "C" fn data_probe_chunk_length(
    mut connp: *mut crate::src::htp_connection_parser::htp_connp_t,
) -> libc::c_int {
    if (*connp).out_current_read_offset - (*connp).out_current_consume_offset
        < 8 as libc::c_int as libc::c_long
    {
        // not enough data so far, consider valid still
        return 1 as libc::c_int;
    }
    let mut data: *mut libc::c_uchar = (*connp)
        .out_current_data
        .offset((*connp).out_current_consume_offset as isize);
    let mut len: size_t =
        ((*connp).out_current_read_offset - (*connp).out_current_consume_offset) as size_t;
    let mut i: size_t = 0 as libc::c_int as size_t;
    while i < len {
        let mut c: libc::c_uchar = *data.offset(i as isize);
        if c as libc::c_int == 0xd as libc::c_int
            || c as libc::c_int == 0xa as libc::c_int
            || c as libc::c_int == 0x20 as libc::c_int
            || c as libc::c_int == 0x9 as libc::c_int
            || c as libc::c_int == 0xb as libc::c_int
            || c as libc::c_int == 0xc as libc::c_int
        {
        } else if *(*__ctype_b_loc()).offset(c as libc::c_int as isize) as libc::c_int
            & _ISdigit as libc::c_int as libc::c_ushort as libc::c_int
            != 0
            || c as libc::c_int >= 'a' as i32 && c as libc::c_int <= 'f' as i32
            || c as libc::c_int >= 'A' as i32 && c as libc::c_int <= 'F' as i32
        {
            // real chunklen char
            return 1 as libc::c_int;
        } else {
            // leading junk, bad
            return 0 as libc::c_int;
        }
        i = i.wrapping_add(1)
    }
    return 1 as libc::c_int;
}

/* *
 * Extracts chunk length.
 *
 * @param[in] connp
 * @returns HTP_OK on state change, HTP_ERROR on error, or HTP_DATA when more data is needed.
 */
#[no_mangle]
pub unsafe extern "C" fn htp_connp_RES_BODY_CHUNKED_LENGTH(
    mut connp: *mut crate::src::htp_connection_parser::htp_connp_t,
) -> htp_status_t {
    loop {
        if (*connp).out_current_read_offset < (*connp).out_current_len {
            (*connp).out_next_byte = *(*connp)
                .out_current_data
                .offset((*connp).out_current_read_offset as isize)
                as libc::c_int;
            (*connp).out_current_read_offset += 1;
            (*connp).out_stream_offset += 1
        } else {
            return 5 as libc::c_int;
        }
        // Have we reached the end of the line? Or is this not chunked after all?
        if !((*connp).out_next_byte == '\n' as i32 || data_probe_chunk_length(connp) == 0) {
            continue;
        }
        let mut data: *mut libc::c_uchar = 0 as *mut libc::c_uchar;
        let mut len: size_t = 0;
        if htp_connp_res_consolidate_data(connp, &mut data, &mut len) != 1 as libc::c_int {
            return -(1 as libc::c_int);
        }
        (*(*connp).out_tx).response_message_len =
            ((*(*connp).out_tx).response_message_len as libc::c_ulong).wrapping_add(len) as int64_t
                as int64_t;
        (*connp).out_chunked_length = htp_parse_chunked_length(data, len);
        // empty chunk length line, lets try to continue
        if (*connp).out_chunked_length == -(1004 as libc::c_int) as libc::c_long {
            continue;
        }
        if (*connp).out_chunked_length < 0 as libc::c_int as libc::c_long {
            // reset out_current_read_offset so htp_connp_RES_BODY_IDENTITY_STREAM_CLOSE
            // doesn't miss the first bytes
            if len > (*connp).out_current_read_offset as size_t {
                (*connp).out_current_read_offset = 0 as libc::c_int as int64_t
            } else {
                (*connp).out_current_read_offset =
                    ((*connp).out_current_read_offset as libc::c_ulong).wrapping_sub(len) as int64_t
                        as int64_t
            }
            (*connp).out_state = Some(
                htp_connp_RES_BODY_IDENTITY_STREAM_CLOSE
                    as unsafe extern "C" fn(
                        _: *mut crate::src::htp_connection_parser::htp_connp_t,
                    ) -> htp_status_t,
            );
            (*(*connp).out_tx).response_transfer_coding = HTP_CODING_IDENTITY;
            htp_log(
                connp,
                b"htp_response.c\x00" as *const u8 as *const libc::c_char,
                421 as libc::c_int,
                HTP_LOG_ERROR,
                0 as libc::c_int,
                b"Response chunk encoding: Invalid chunk length: %ld\x00" as *const u8
                    as *const libc::c_char,
                (*connp).out_chunked_length,
            );
            return 1 as libc::c_int;
        }
        htp_connp_res_clear_buffer(connp);
        // Handle chunk length
        if (*connp).out_chunked_length > 0 as libc::c_int as libc::c_long {
            // More data available
            (*connp).out_state = Some(
                htp_connp_RES_BODY_CHUNKED_DATA
                    as unsafe extern "C" fn(
                        _: *mut crate::src::htp_connection_parser::htp_connp_t,
                    ) -> htp_status_t,
            )
        } else if (*connp).out_chunked_length == 0 as libc::c_int as libc::c_long {
            // End of data
            (*connp).out_state = Some(
                htp_connp_RES_HEADERS
                    as unsafe extern "C" fn(
                        _: *mut crate::src::htp_connection_parser::htp_connp_t,
                    ) -> htp_status_t,
            );
            (*(*connp).out_tx).response_progress = HTP_RESPONSE_TRAILER
        }
        return 1 as libc::c_int;
    }
}

/* *
 * Processes an identity response body of known length.
 *
 * @param[in] connp
 * @returns HTP_OK on state change, HTP_ERROR on error, or HTP_DATA when more data is needed.
 */
#[no_mangle]
pub unsafe extern "C" fn htp_connp_RES_BODY_IDENTITY_CL_KNOWN(
    mut connp: *mut crate::src::htp_connection_parser::htp_connp_t,
) -> htp_status_t {
    let mut bytes_to_consume: size_t = 0;
    // Determine how many bytes we can consume.
    if (*connp).out_current_len - (*connp).out_current_read_offset >= (*connp).out_body_data_left {
        bytes_to_consume = (*connp).out_body_data_left as size_t
    } else {
        bytes_to_consume = ((*connp).out_current_len - (*connp).out_current_read_offset) as size_t
    }
    if (*connp).out_status as libc::c_uint == HTP_STREAM_CLOSED as libc::c_int as libc::c_uint {
        (*connp).out_state = Some(
            htp_connp_RES_FINALIZE
                as unsafe extern "C" fn(
                    _: *mut crate::src::htp_connection_parser::htp_connp_t,
                ) -> htp_status_t,
        );
        // Sends close signal to decompressors
        let mut rc: htp_status_t = htp_tx_res_process_body_data_ex(
            (*connp).out_tx,
            0 as *const libc::c_void,
            0 as libc::c_int as size_t,
        );
        return rc;
    }
    if bytes_to_consume == 0 as libc::c_int as libc::c_ulong {
        return 2 as libc::c_int;
    }
    // Consume the data.
    let mut rc_0: htp_status_t = htp_tx_res_process_body_data_ex(
        (*connp).out_tx,
        (*connp)
            .out_current_data
            .offset((*connp).out_current_read_offset as isize) as *const libc::c_void,
        bytes_to_consume,
    );
    if rc_0 != 1 as libc::c_int {
        return rc_0;
    }
    // Adjust the counters.
    (*connp).out_current_read_offset = ((*connp).out_current_read_offset as libc::c_ulong)
        .wrapping_add(bytes_to_consume) as int64_t
        as int64_t;
    (*connp).out_current_consume_offset = ((*connp).out_current_consume_offset as libc::c_ulong)
        .wrapping_add(bytes_to_consume) as int64_t
        as int64_t;
    (*connp).out_stream_offset = ((*connp).out_stream_offset as libc::c_ulong)
        .wrapping_add(bytes_to_consume) as int64_t as int64_t;
    (*connp).out_body_data_left = ((*connp).out_body_data_left as libc::c_ulong)
        .wrapping_sub(bytes_to_consume) as int64_t as int64_t;
    // Have we seen the entire response body?
    if (*connp).out_body_data_left == 0 as libc::c_int as libc::c_long {
        (*connp).out_state = Some(
            htp_connp_RES_FINALIZE
                as unsafe extern "C" fn(
                    _: *mut crate::src::htp_connection_parser::htp_connp_t,
                ) -> htp_status_t,
        );
        // Tells decompressors to output partially decompressed data
        rc_0 = htp_tx_res_process_body_data_ex(
            (*connp).out_tx,
            0 as *const libc::c_void,
            0 as libc::c_int as size_t,
        );
        return rc_0;
    }
    return 2 as libc::c_int;
}

/* *
 * Processes identity response body of unknown length. In this case, we assume the
 * response body consumes all data until the end of the stream.
 *
 * @param[in] connp
 * @returns HTP_OK on state change, HTP_ERROR on error, or HTP_DATA when more data is needed.
 */
#[no_mangle]
pub unsafe extern "C" fn htp_connp_RES_BODY_IDENTITY_STREAM_CLOSE(
    mut connp: *mut crate::src::htp_connection_parser::htp_connp_t,
) -> htp_status_t {
    // Consume all data from the input buffer.
    let mut bytes_to_consume: size_t =
        ((*connp).out_current_len - (*connp).out_current_read_offset) as size_t;
    if bytes_to_consume != 0 as libc::c_int as libc::c_ulong {
        let mut rc: htp_status_t = htp_tx_res_process_body_data_ex(
            (*connp).out_tx,
            (*connp)
                .out_current_data
                .offset((*connp).out_current_read_offset as isize)
                as *const libc::c_void,
            bytes_to_consume,
        );
        if rc != 1 as libc::c_int {
            return rc;
        }
        // Adjust the counters.
        (*connp).out_current_read_offset = ((*connp).out_current_read_offset as libc::c_ulong)
            .wrapping_add(bytes_to_consume) as int64_t
            as int64_t;
        (*connp).out_current_consume_offset = ((*connp).out_current_consume_offset as libc::c_ulong)
            .wrapping_add(bytes_to_consume) as int64_t
            as int64_t;
        (*connp).out_stream_offset = ((*connp).out_stream_offset as libc::c_ulong)
            .wrapping_add(bytes_to_consume) as int64_t
            as int64_t
    }
    // Have we seen the entire response body?
    if (*connp).out_status as libc::c_uint == HTP_STREAM_CLOSED as libc::c_int as libc::c_uint {
        (*connp).out_state = Some(
            htp_connp_RES_FINALIZE
                as unsafe extern "C" fn(
                    _: *mut crate::src::htp_connection_parser::htp_connp_t,
                ) -> htp_status_t,
        );
        return 1 as libc::c_int;
    }
    return 2 as libc::c_int;
}

/* *
 * Determines presence (and encoding) of a response body.
 *
 * @param[in] connp
 * @returns HTP_OK on state change, HTP_ERROR on error, or HTP_DATA when more data is needed.
 */
#[no_mangle]
pub unsafe extern "C" fn htp_connp_RES_BODY_DETERMINE(
    mut connp: *mut crate::src::htp_connection_parser::htp_connp_t,
) -> htp_status_t {
    // If the request uses the CONNECT method, then not only are we
    // to assume there's no body, but we need to ignore all
    // subsequent data in the stream.
    if (*(*connp).out_tx).request_method_number as libc::c_uint
        == HTP_M_CONNECT as libc::c_int as libc::c_uint
    {
        if (*(*connp).out_tx).response_status_number >= 200 as libc::c_int
            && (*(*connp).out_tx).response_status_number <= 299 as libc::c_int
        {
            // This is a successful CONNECT stream, which means
            // we need to switch into tunneling mode: on the
            // request side we'll now probe the tunnel data to see
            // if we need to parse or ignore it. So on the response
            // side we wrap up the tx and wait.
            (*connp).out_state = Some(
                htp_connp_RES_FINALIZE
                    as unsafe extern "C" fn(
                        _: *mut crate::src::htp_connection_parser::htp_connp_t,
                    ) -> htp_status_t,
            );
            // we may have response headers
            let mut rc: htp_status_t = htp_tx_state_response_headers((*connp).out_tx);
            return rc;
        } else {
            if (*(*connp).out_tx).response_status_number == 407 as libc::c_int {
                // proxy telling us to auth
                (*connp).in_status = HTP_STREAM_DATA
            } else {
                // This is a failed CONNECT stream, which means that
                // we can unblock request parsing
                (*connp).in_status = HTP_STREAM_DATA;
                // We are going to continue processing this transaction,
                // adding a note for ourselves to stop at the end (because
                // we don't want to see the beginning of a new transaction).
                (*connp).out_data_other_at_tx_end = 1 as libc::c_int as libc::c_uint
            }
        }
    }
    let mut cl: *mut crate::src::htp_transaction::htp_header_t = htp_table_get_c(
        (*(*connp).out_tx).response_headers,
        b"content-length\x00" as *const u8 as *const libc::c_char,
    )
        as *mut crate::src::htp_transaction::htp_header_t;
    let mut te: *mut crate::src::htp_transaction::htp_header_t = htp_table_get_c(
        (*(*connp).out_tx).response_headers,
        b"transfer-encoding\x00" as *const u8 as *const libc::c_char,
    )
        as *mut crate::src::htp_transaction::htp_header_t;
    // Check for "101 Switching Protocol" response.
    // If it's seen, it means that traffic after empty line following headers
    // is no longer HTTP. We can treat it similarly to CONNECT.
    // Unlike CONNECT, however, upgrades from HTTP to HTTP seem
    // rather unlikely, so don't try to probe tunnel for nested HTTP,
    // and switch to tunnel mode right away.
    if (*(*connp).out_tx).response_status_number == 101 as libc::c_int {
        if te.is_null() && cl.is_null() {
            (*connp).out_state = Some(
                htp_connp_RES_FINALIZE
                    as unsafe extern "C" fn(
                        _: *mut crate::src::htp_connection_parser::htp_connp_t,
                    ) -> htp_status_t,
            );
            (*connp).in_status = HTP_STREAM_TUNNEL;
            (*connp).out_status = HTP_STREAM_TUNNEL;
            // we may have response headers
            let mut rc_0: htp_status_t = htp_tx_state_response_headers((*connp).out_tx);
            return rc_0;
        } else {
            htp_log(
                connp,
                b"htp_response.c\x00" as *const u8 as *const libc::c_char,
                581 as libc::c_int,
                HTP_LOG_WARNING,
                0 as libc::c_int,
                b"Switching Protocol with Content-Length\x00" as *const u8 as *const libc::c_char,
            );
        }
    }
    // Check for an interim "100 Continue" response. Ignore it if found, and revert back to RES_LINE.
    if (*(*connp).out_tx).response_status_number == 100 as libc::c_int
        && te.is_null()
        && cl.is_null()
    {
        if (*(*connp).out_tx).seen_100continue != 0 as libc::c_int {
            htp_log(
                connp,
                b"htp_response.c\x00" as *const u8 as *const libc::c_char,
                588 as libc::c_int,
                HTP_LOG_ERROR,
                0 as libc::c_int,
                b"Already seen 100-Continue.\x00" as *const u8 as *const libc::c_char,
            );
            return -(1 as libc::c_int);
        }
        // Ignore any response headers seen so far.
        let mut h: *mut crate::src::htp_transaction::htp_header_t =
            0 as *mut crate::src::htp_transaction::htp_header_t;
        let mut i: size_t = 0 as libc::c_int as size_t;
        let mut n: size_t = htp_table_size((*(*connp).out_tx).response_headers);
        while i < n {
            h = htp_table_get_index((*(*connp).out_tx).response_headers, i, 0 as *mut *mut bstr)
                as *mut crate::src::htp_transaction::htp_header_t;
            bstr_free((*h).name);
            bstr_free((*h).value);
            free(h as *mut libc::c_void);
            i = i.wrapping_add(1)
        }
        htp_table_clear((*(*connp).out_tx).response_headers);
        // Expecting to see another response line next.
        (*connp).out_state = Some(
            htp_connp_RES_LINE
                as unsafe extern "C" fn(
                    _: *mut crate::src::htp_connection_parser::htp_connp_t,
                ) -> htp_status_t,
        );
        (*(*connp).out_tx).response_progress = HTP_RESPONSE_LINE;
        (*(*connp).out_tx).seen_100continue += 1;
        return 1 as libc::c_int;
    }
    // 1. Any response message which MUST NOT include a message-body
    //  (such as the 1xx, 204, and 304 responses and any response to a HEAD
    //  request) is always terminated by the first empty line after the
    //  header fields, regardless of the entity-header fields present in the
    //  message.
    if (*(*connp).out_tx).request_method_number as libc::c_uint
        == HTP_M_HEAD as libc::c_int as libc::c_uint
    {
        // There's no response body whatsoever
        (*(*connp).out_tx).response_transfer_coding = HTP_CODING_NO_BODY;
        (*connp).out_state = Some(
            htp_connp_RES_FINALIZE
                as unsafe extern "C" fn(
                    _: *mut crate::src::htp_connection_parser::htp_connp_t,
                ) -> htp_status_t,
        )
    } else if (*(*connp).out_tx).response_status_number >= 100 as libc::c_int
        && (*(*connp).out_tx).response_status_number <= 199 as libc::c_int
        || (*(*connp).out_tx).response_status_number == 204 as libc::c_int
        || (*(*connp).out_tx).response_status_number == 304 as libc::c_int
    {
        // There should be no response body
        // but browsers interpret content sent by the server as such
        if te.is_null() && cl.is_null() {
            (*(*connp).out_tx).response_transfer_coding = HTP_CODING_NO_BODY;
            (*connp).out_state = Some(
                htp_connp_RES_FINALIZE
                    as unsafe extern "C" fn(
                        _: *mut crate::src::htp_connection_parser::htp_connp_t,
                    ) -> htp_status_t,
            )
        } else {
            htp_log(
                connp,
                b"htp_response.c\x00" as *const u8 as *const libc::c_char,
                629 as libc::c_int,
                HTP_LOG_WARNING,
                0 as libc::c_int,
                b"Unexpected Response body\x00" as *const u8 as *const libc::c_char,
            );
        }
    }
    // Hack condition to check that we do not assume "no body"
    if (*connp).out_state
        != Some(
            htp_connp_RES_FINALIZE
                as unsafe extern "C" fn(
                    _: *mut crate::src::htp_connection_parser::htp_connp_t,
                ) -> htp_status_t,
        )
    {
        // We have a response body
        let mut ct: *mut crate::src::htp_transaction::htp_header_t = htp_table_get_c(
            (*(*connp).out_tx).response_headers,
            b"content-type\x00" as *const u8 as *const libc::c_char,
        )
            as *mut crate::src::htp_transaction::htp_header_t;
        if !ct.is_null() {
            (*(*connp).out_tx).response_content_type = bstr_dup_lower((*ct).value);
            if (*(*connp).out_tx).response_content_type.is_null() {
                return -(1 as libc::c_int);
            }
            // Ignore parameters
            let mut data: *mut libc::c_uchar = if (*(*(*connp).out_tx).response_content_type)
                .realptr
                .is_null()
            {
                ((*(*connp).out_tx).response_content_type as *mut libc::c_uchar)
                    .offset(::std::mem::size_of::<bstr>() as libc::c_ulong as isize)
            } else {
                (*(*(*connp).out_tx).response_content_type).realptr
            };
            let mut len: size_t = (*(*ct).value).len;
            let mut newlen: size_t = 0 as libc::c_int as size_t;
            while newlen < len {
                // TODO Some platforms may do things differently here.
                if htp_is_space(*data.offset(newlen as isize) as libc::c_int) != 0
                    || *data.offset(newlen as isize) as libc::c_int == ';' as i32
                {
                    bstr_adjust_len((*(*connp).out_tx).response_content_type, newlen);
                    break;
                } else {
                    newlen = newlen.wrapping_add(1)
                }
            }
        }
        // 2. If a Transfer-Encoding header field (section 14.40) is present and
        //   indicates that the "chunked" transfer coding has been applied, then
        //   the length is defined by the chunked encoding (section 3.6).
        if !te.is_null()
            && bstr_index_of_c_nocasenorzero(
                (*te).value,
                b"chunked\x00" as *const u8 as *const libc::c_char,
            ) != -(1 as libc::c_int)
        {
            if bstr_cmp_c_nocase(
                (*te).value,
                b"chunked\x00" as *const u8 as *const libc::c_char,
            ) != 0 as libc::c_int
            {
                htp_log(
                    connp,
                    b"htp_response.c\x00" as *const u8 as *const libc::c_char,
                    660 as libc::c_int,
                    HTP_LOG_WARNING,
                    0 as libc::c_int,
                    b"Transfer-encoding has abnormal chunked value\x00" as *const u8
                        as *const libc::c_char,
                ); // 3. If a Content-Length header field (section 14.14) is present, its
            }
            // spec says chunked is HTTP/1.1 only, but some browsers accept it
            // with 1.0 as well
            if (*(*connp).out_tx).response_protocol_number < 101 as libc::c_int {
                htp_log(
                    connp,
                    b"htp_response.c\x00" as *const u8 as *const libc::c_char,
                    667 as libc::c_int,
                    HTP_LOG_WARNING,
                    0 as libc::c_int,
                    b"Chunked transfer-encoding on HTTP/0.9 or HTTP/1.0\x00" as *const u8
                        as *const libc::c_char,
                );
            }
            // If the T-E header is present we are going to use it.
            (*(*connp).out_tx).response_transfer_coding = HTP_CODING_CHUNKED;
            // We are still going to check for the presence of C-L
            if !cl.is_null() {
                // This is a violation of the RFC
                (*(*connp).out_tx).flags = ((*(*connp).out_tx).flags as libc::c_ulonglong
                    | 0x100 as libc::c_ulonglong)
                    as uint64_t
            }
            (*connp).out_state = Some(
                htp_connp_RES_BODY_CHUNKED_LENGTH
                    as unsafe extern "C" fn(
                        _: *mut crate::src::htp_connection_parser::htp_connp_t,
                    ) -> htp_status_t,
            );
            (*(*connp).out_tx).response_progress = HTP_RESPONSE_BODY
        } else if !cl.is_null() {
            //   value in bytes represents the length of the message-body.
            // We know the exact length
            (*(*connp).out_tx).response_transfer_coding = HTP_CODING_IDENTITY;
            // Check for multiple C-L headers
            if (*cl).flags as libc::c_ulonglong & 0x20 as libc::c_ulonglong != 0 {
                (*(*connp).out_tx).flags = ((*(*connp).out_tx).flags as libc::c_ulonglong
                    | 0x100 as libc::c_ulonglong)
                    as uint64_t
            }
            // Get body length
            (*(*connp).out_tx).response_content_length =
                htp_parse_content_length((*cl).value, connp);
            if (*(*connp).out_tx).response_content_length < 0 as libc::c_int as libc::c_long {
                htp_log(
                    connp,
                    b"htp_response.c\x00" as *const u8 as *const libc::c_char,
                    696 as libc::c_int,
                    HTP_LOG_ERROR,
                    0 as libc::c_int,
                    b"Invalid C-L field in response: %ld\x00" as *const u8 as *const libc::c_char,
                    (*(*connp).out_tx).response_content_length,
                );
                return -(1 as libc::c_int);
            } else {
                (*connp).out_content_length = (*(*connp).out_tx).response_content_length;
                (*connp).out_body_data_left = (*connp).out_content_length;
                if (*connp).out_content_length != 0 as libc::c_int as libc::c_long {
                    (*connp).out_state = Some(
                        htp_connp_RES_BODY_IDENTITY_CL_KNOWN
                            as unsafe extern "C" fn(
                                _: *mut crate::src::htp_connection_parser::htp_connp_t,
                            ) -> htp_status_t,
                    );
                    (*(*connp).out_tx).response_progress = HTP_RESPONSE_BODY
                } else {
                    (*connp).out_state = Some(
                        htp_connp_RES_FINALIZE
                            as unsafe extern "C" fn(
                                _: *mut crate::src::htp_connection_parser::htp_connp_t,
                            ) -> htp_status_t,
                    )
                }
            }
        } else {
            // 4. If the message uses the media type "multipart/byteranges", which is
            //   self-delimiting, then that defines the length. This media type MUST
            //   NOT be used unless the sender knows that the recipient can parse it;
            //   the presence in a request of a Range header with multiple byte-range
            //   specifiers implies that the client can parse multipart/byteranges
            //   responses.
            if !ct.is_null() {
                // TODO Handle multipart/byteranges
                if bstr_index_of_c_nocase(
                    (*ct).value,
                    b"multipart/byteranges\x00" as *const u8 as *const libc::c_char,
                ) != -(1 as libc::c_int)
                {
                    htp_log(
                        connp,
                        b"htp_response.c\x00" as *const u8 as *const libc::c_char,
                        720 as libc::c_int,
                        HTP_LOG_ERROR,
                        0 as libc::c_int,
                        b"C-T multipart/byteranges in responses not supported\x00" as *const u8
                            as *const libc::c_char,
                    );
                    return -(1 as libc::c_int);
                }
            }
            // 5. By the server closing the connection. (Closing the connection
            //   cannot be used to indicate the end of a request body, since that
            //   would leave no possibility for the server to send back a response.)
            (*connp).out_state = Some(
                htp_connp_RES_BODY_IDENTITY_STREAM_CLOSE
                    as unsafe extern "C" fn(
                        _: *mut crate::src::htp_connection_parser::htp_connp_t,
                    ) -> htp_status_t,
            );
            (*(*connp).out_tx).response_transfer_coding = HTP_CODING_IDENTITY;
            (*(*connp).out_tx).response_progress = HTP_RESPONSE_BODY;
            (*connp).out_body_data_left = -(1 as libc::c_int) as int64_t
        }
    }
    // NOTE We do not need to check for short-style HTTP/0.9 requests here because
    //      that is done earlier, before response line parsing begins
    let mut rc_1: htp_status_t = htp_tx_state_response_headers((*connp).out_tx);
    if rc_1 != 1 as libc::c_int {
        return rc_1;
    }
    return 1 as libc::c_int;
}

/* *
 * Parses response headers.
 *
 * @param[in] connp
 * @returns HTP_OK on state change, HTP_ERROR on error, or HTP_DATA when more data is needed.
 */
#[no_mangle]
pub unsafe extern "C" fn htp_connp_RES_HEADERS(
    mut connp: *mut crate::src::htp_connection_parser::htp_connp_t,
) -> htp_status_t {
    let mut endwithcr: libc::c_int = 0;
    let mut lfcrending: libc::c_int = 0 as libc::c_int;
    loop {
        if (*connp).out_status as libc::c_uint == HTP_STREAM_CLOSED as libc::c_int as libc::c_uint {
            // Finalize sending raw trailer data.
            let mut rc: htp_status_t = htp_connp_res_receiver_finalize_clear(connp);
            if rc != 1 as libc::c_int {
                return rc;
            }
            // Run hook response_TRAILER.
            rc = htp_hook_run_all(
                (*(*connp).cfg).hook_response_trailer,
                (*connp).out_tx as *mut libc::c_void,
            );
            if rc != 1 as libc::c_int {
                return rc;
            }
            (*connp).out_state = Some(
                htp_connp_RES_FINALIZE
                    as unsafe extern "C" fn(
                        _: *mut crate::src::htp_connection_parser::htp_connp_t,
                    ) -> htp_status_t,
            );
            return 1 as libc::c_int;
        }
        if (*connp).out_current_read_offset < (*connp).out_current_len {
            (*connp).out_next_byte = *(*connp)
                .out_current_data
                .offset((*connp).out_current_read_offset as isize)
                as libc::c_int;
            (*connp).out_current_read_offset += 1;
            (*connp).out_stream_offset += 1
        } else {
            return 5 as libc::c_int;
        }
        // Have we reached the end of the line?
        if (*connp).out_next_byte != '\n' as i32 && (*connp).out_next_byte != '\r' as i32 {
            lfcrending = 0 as libc::c_int
        } else {
            endwithcr = 0 as libc::c_int;
            if (*connp).out_next_byte == '\r' as i32 {
                if (*connp).out_current_read_offset >= (*connp).out_current_len {
                    (*connp).out_next_byte = -(1 as libc::c_int)
                } else {
                    (*connp).out_next_byte = *(*connp)
                        .out_current_data
                        .offset((*connp).out_current_read_offset as isize)
                        as libc::c_int
                }
                if (*connp).out_next_byte == -(1 as libc::c_int) {
                    return 5 as libc::c_int;
                } else {
                    if (*connp).out_next_byte == '\n' as i32 {
                        if (*connp).out_current_read_offset < (*connp).out_current_len {
                            (*connp).out_next_byte = *(*connp)
                                .out_current_data
                                .offset((*connp).out_current_read_offset as isize)
                                as libc::c_int;
                            (*connp).out_current_read_offset += 1;
                            (*connp).out_stream_offset += 1
                        } else {
                            return 5 as libc::c_int;
                        }
                        if lfcrending != 0 {
                            // Handling LFCRCRLFCRLF
                            // These 6 characters mean only 2 end of lines
                            if (*connp).out_current_read_offset >= (*connp).out_current_len {
                                (*connp).out_next_byte = -(1 as libc::c_int)
                            } else {
                                (*connp).out_next_byte = *(*connp)
                                    .out_current_data
                                    .offset((*connp).out_current_read_offset as isize)
                                    as libc::c_int
                            }
                            if (*connp).out_next_byte == '\r' as i32 {
                                if (*connp).out_current_read_offset < (*connp).out_current_len {
                                    (*connp).out_next_byte = *(*connp)
                                        .out_current_data
                                        .offset((*connp).out_current_read_offset as isize)
                                        as libc::c_int;
                                    (*connp).out_current_read_offset += 1;
                                    (*connp).out_stream_offset += 1
                                } else {
                                    return 5 as libc::c_int;
                                }
                                (*connp).out_current_consume_offset += 1;
                                if (*connp).out_current_read_offset >= (*connp).out_current_len {
                                    (*connp).out_next_byte = -(1 as libc::c_int)
                                } else {
                                    (*connp).out_next_byte = *(*connp)
                                        .out_current_data
                                        .offset((*connp).out_current_read_offset as isize)
                                        as libc::c_int
                                }
                                if (*connp).out_next_byte == '\n' as i32 {
                                    if (*connp).out_current_read_offset < (*connp).out_current_len {
                                        (*connp).out_next_byte = *(*connp)
                                            .out_current_data
                                            .offset((*connp).out_current_read_offset as isize)
                                            as libc::c_int;
                                        (*connp).out_current_read_offset += 1;
                                        (*connp).out_stream_offset += 1
                                    } else {
                                        return 5 as libc::c_int;
                                    }
                                    (*connp).out_current_consume_offset += 1;
                                    htp_log(
                                        connp,
                                        b"htp_response.c\x00" as *const u8 as *const libc::c_char,
                                        792 as libc::c_int,
                                        HTP_LOG_WARNING,
                                        0 as libc::c_int,
                                        b"Weird response end of lines mix\x00" as *const u8
                                            as *const libc::c_char,
                                    );
                                }
                            }
                        }
                    } else if (*connp).out_next_byte == '\r' as i32 {
                        continue;
                    }
                    lfcrending = 0 as libc::c_int;
                    endwithcr = 1 as libc::c_int
                }
            } else {
                // connp->out_next_byte == LF
                if (*connp).out_current_read_offset >= (*connp).out_current_len {
                    (*connp).out_next_byte = -(1 as libc::c_int)
                } else {
                    (*connp).out_next_byte = *(*connp)
                        .out_current_data
                        .offset((*connp).out_current_read_offset as isize)
                        as libc::c_int
                }
                lfcrending = 0 as libc::c_int;
                if (*connp).out_next_byte == '\r' as i32 {
                    // hanldes LF-CR sequence as end of line
                    if (*connp).out_current_read_offset < (*connp).out_current_len {
                        (*connp).out_next_byte = *(*connp)
                            .out_current_data
                            .offset((*connp).out_current_read_offset as isize)
                            as libc::c_int;
                        (*connp).out_current_read_offset += 1;
                        (*connp).out_stream_offset += 1
                    } else {
                        return 5 as libc::c_int;
                    }
                    lfcrending = 1 as libc::c_int
                }
            }
            let mut data: *mut libc::c_uchar = 0 as *mut libc::c_uchar;
            let mut len: size_t = 0;
            if htp_connp_res_consolidate_data(connp, &mut data, &mut len) != 1 as libc::c_int {
                return -(1 as libc::c_int);
            }
            // CRCRLF is not an empty line
            if endwithcr != 0 && len < 2 as libc::c_int as libc::c_ulong {
                continue;
            }
            // Should we terminate headers?
            if htp_connp_is_line_terminator(connp, data, len) != 0 {
                // Parse previous header, if any.
                if !(*connp).out_header.is_null() {
                    if (*(*connp).cfg)
                        .process_response_header
                        .expect("non-null function pointer")(
                        connp,
                        if (*(*connp).out_header).realptr.is_null() {
                            ((*connp).out_header as *mut libc::c_uchar)
                                .offset(::std::mem::size_of::<bstr>() as libc::c_ulong as isize)
                        } else {
                            (*(*connp).out_header).realptr
                        },
                        (*(*connp).out_header).len,
                    ) != 1 as libc::c_int
                    {
                        return -(1 as libc::c_int);
                    }
                    bstr_free((*connp).out_header);
                    (*connp).out_header = 0 as *mut bstr
                }
                htp_connp_res_clear_buffer(connp);
                // We've seen all response headers.
                if (*(*connp).out_tx).response_progress as libc::c_uint
                    == HTP_RESPONSE_HEADERS as libc::c_int as libc::c_uint
                {
                    // Response headers.
                    // The next step is to determine if this response has a body.
                    (*connp).out_state = Some(
                        htp_connp_RES_BODY_DETERMINE
                            as unsafe extern "C" fn(
                                _: *mut crate::src::htp_connection_parser::htp_connp_t,
                            ) -> htp_status_t,
                    )
                } else {
                    // Response trailer.
                    // Finalize sending raw trailer data.
                    let mut rc_0: htp_status_t = htp_connp_res_receiver_finalize_clear(connp);
                    if rc_0 != 1 as libc::c_int {
                        return rc_0;
                    }
                    // Run hook response_TRAILER.
                    rc_0 = htp_hook_run_all(
                        (*(*connp).cfg).hook_response_trailer,
                        (*connp).out_tx as *mut libc::c_void,
                    );
                    if rc_0 != 1 as libc::c_int {
                        return rc_0;
                    }
                    // The next step is to finalize this response.
                    (*connp).out_state = Some(
                        htp_connp_RES_FINALIZE
                            as unsafe extern "C" fn(
                                _: *mut crate::src::htp_connection_parser::htp_connp_t,
                            ) -> htp_status_t,
                    )
                }
                return 1 as libc::c_int;
            }
            htp_chomp(data, &mut len);
            // Check for header folding.
            if htp_connp_is_line_folded(data, len) == 0 as libc::c_int {
                // New header line.
                // Parse previous header, if any.
                if !(*connp).out_header.is_null() {
                    if (*(*connp).cfg)
                        .process_response_header
                        .expect("non-null function pointer")(
                        connp,
                        if (*(*connp).out_header).realptr.is_null() {
                            ((*connp).out_header as *mut libc::c_uchar)
                                .offset(::std::mem::size_of::<bstr>() as libc::c_ulong as isize)
                        } else {
                            (*(*connp).out_header).realptr
                        },
                        (*(*connp).out_header).len,
                    ) != 1 as libc::c_int
                    {
                        return -(1 as libc::c_int);
                    }
                    bstr_free((*connp).out_header);
                    (*connp).out_header = 0 as *mut bstr
                }
                if (*connp).out_current_read_offset >= (*connp).out_current_len {
                    (*connp).out_next_byte = -(1 as libc::c_int)
                } else {
                    (*connp).out_next_byte = *(*connp)
                        .out_current_data
                        .offset((*connp).out_current_read_offset as isize)
                        as libc::c_int
                }
                if htp_is_folding_char((*connp).out_next_byte) == 0 as libc::c_int {
                    // Because we know this header is not folded, we can process the buffer straight away.
                    if (*(*connp).cfg)
                        .process_response_header
                        .expect("non-null function pointer")(connp, data, len)
                        != 1 as libc::c_int
                    {
                        return -(1 as libc::c_int);
                    }
                } else {
                    // Keep the partial header data for parsing later.
                    (*connp).out_header = bstr_dup_mem(data as *const libc::c_void, len);
                    if (*connp).out_header.is_null() {
                        return -(1 as libc::c_int);
                    }
                }
            } else if (*connp).out_header.is_null() {
                // Folding; check that there's a previous header line to add to.
                // Invalid folding.
                // Warn only once per transaction.
                if (*(*connp).out_tx).flags as libc::c_ulonglong & 0x200 as libc::c_ulonglong == 0 {
                    (*(*connp).out_tx).flags = ((*(*connp).out_tx).flags as libc::c_ulonglong
                        | 0x200 as libc::c_ulonglong)
                        as uint64_t;
                    htp_log(
                        connp,
                        b"htp_response.c\x00" as *const u8 as *const libc::c_char,
                        899 as libc::c_int,
                        HTP_LOG_WARNING,
                        0 as libc::c_int,
                        b"Invalid response field folding\x00" as *const u8 as *const libc::c_char,
                    );
                }
                // Keep the header data for parsing later.
                (*connp).out_header = bstr_dup_mem(data as *const libc::c_void, len);
                if (*connp).out_header.is_null() {
                    return -(1 as libc::c_int);
                }
            } else {
                let mut colon_pos: size_t = 0 as libc::c_int as size_t;
                while colon_pos < len
                    && *data.offset(colon_pos as isize) as libc::c_int != ':' as i32
                {
                    colon_pos = colon_pos.wrapping_add(1)
                }
                if colon_pos < len
                    && bstr_chr((*connp).out_header, ':' as i32) >= 0 as libc::c_int
                    && (*(*connp).out_tx).response_protocol_number == 101 as libc::c_int
                {
                    // Warn only once per transaction.
                    if (*(*connp).out_tx).flags as libc::c_ulonglong & 0x200 as libc::c_ulonglong
                        == 0
                    {
                        (*(*connp).out_tx).flags = ((*(*connp).out_tx).flags as libc::c_ulonglong
                            | 0x200 as libc::c_ulonglong)
                            as uint64_t;
                        htp_log(
                            connp,
                            b"htp_response.c\x00" as *const u8 as *const libc::c_char,
                            915 as libc::c_int,
                            HTP_LOG_WARNING,
                            0 as libc::c_int,
                            b"Invalid response field folding\x00" as *const u8
                                as *const libc::c_char,
                        );
                    }
                    if (*(*connp).cfg)
                        .process_response_header
                        .expect("non-null function pointer")(
                        connp,
                        if (*(*connp).out_header).realptr.is_null() {
                            ((*connp).out_header as *mut libc::c_uchar)
                                .offset(::std::mem::size_of::<bstr>() as libc::c_ulong as isize)
                        } else {
                            (*(*connp).out_header).realptr
                        },
                        (*(*connp).out_header).len,
                    ) != 1 as libc::c_int
                    {
                        return -(1 as libc::c_int);
                    }
                    bstr_free((*connp).out_header);
                    (*connp).out_header = bstr_dup_mem(
                        data.offset(1 as libc::c_int as isize) as *const libc::c_void,
                        len.wrapping_sub(1 as libc::c_int as libc::c_ulong),
                    );
                    if (*connp).out_header.is_null() {
                        return -(1 as libc::c_int);
                    }
                } else {
                    // Add to the existing header.
                    let mut new_out_header: *mut bstr =
                        bstr_add_mem((*connp).out_header, data as *const libc::c_void, len);
                    if new_out_header.is_null() {
                        return -(1 as libc::c_int);
                    }
                    (*connp).out_header = new_out_header
                }
            }
            htp_connp_res_clear_buffer(connp);
        }
    }
}

/* *
 * Parses response line.
 *
 * @param[in] connp
 * @returns HTP_OK on state change, HTP_ERROR on error, or HTP_DATA when more data is needed.
 */
#[no_mangle]
pub unsafe extern "C" fn htp_connp_RES_LINE(
    mut connp: *mut crate::src::htp_connection_parser::htp_connp_t,
) -> htp_status_t {
    loop {
        // Don't try to get more data if the stream is closed. If we do, we'll return, asking for more data.
        if (*connp).out_status as libc::c_uint != HTP_STREAM_CLOSED as libc::c_int as libc::c_uint {
            // Get one byte
            if (*connp).out_current_read_offset < (*connp).out_current_len {
                (*connp).out_next_byte = *(*connp)
                    .out_current_data
                    .offset((*connp).out_current_read_offset as isize)
                    as libc::c_int;
                (*connp).out_current_read_offset += 1;
                (*connp).out_stream_offset += 1
            } else {
                return 5 as libc::c_int;
            }
        }
        // Have we reached the end of the line? We treat stream closure as end of line in
        // order to handle the case when the first line of the response is actually response body
        // (and we wish it processed as such).
        if (*connp).out_next_byte == '\r' as i32 {
            if (*connp).out_current_read_offset >= (*connp).out_current_len {
                (*connp).out_next_byte = -(1 as libc::c_int)
            } else {
                (*connp).out_next_byte = *(*connp)
                    .out_current_data
                    .offset((*connp).out_current_read_offset as isize)
                    as libc::c_int
            }
            if (*connp).out_next_byte == -(1 as libc::c_int) {
                return 5 as libc::c_int;
            } else {
                if (*connp).out_next_byte == '\n' as i32 {
                    continue;
                }
                (*connp).out_next_byte = '\n' as i32
            }
        }
        if (*connp).out_next_byte == '\n' as i32
            || (*connp).out_status as libc::c_uint
                == HTP_STREAM_CLOSED as libc::c_int as libc::c_uint
        {
            let mut data: *mut libc::c_uchar = 0 as *mut libc::c_uchar;
            let mut len: size_t = 0;
            if htp_connp_res_consolidate_data(connp, &mut data, &mut len) != 1 as libc::c_int {
                return -(1 as libc::c_int);
            }
            // Is this a line that should be ignored?
            if htp_connp_is_line_ignorable(connp, data, len) != 0 {
                if (*connp).out_status as libc::c_uint
                    == HTP_STREAM_CLOSED as libc::c_int as libc::c_uint
                {
                    (*connp).out_state = Some(
                        htp_connp_RES_FINALIZE
                            as unsafe extern "C" fn(
                                _: *mut crate::src::htp_connection_parser::htp_connp_t,
                            ) -> htp_status_t,
                    )
                }
                // We have an empty/whitespace line, which we'll note, ignore and move on
                (*(*connp).out_tx).response_ignored_lines =
                    (*(*connp).out_tx).response_ignored_lines.wrapping_add(1);
                // TODO How many lines are we willing to accept?
                // Start again
                htp_connp_res_clear_buffer(connp);
                return 1 as libc::c_int;
            }
            // Deallocate previous response line allocations, which we would have on a 100 response.
            if !(*(*connp).out_tx).response_line.is_null() {
                bstr_free((*(*connp).out_tx).response_line);
                (*(*connp).out_tx).response_line = 0 as *mut bstr
            }
            if !(*(*connp).out_tx).response_protocol.is_null() {
                bstr_free((*(*connp).out_tx).response_protocol);
                (*(*connp).out_tx).response_protocol = 0 as *mut bstr
            }
            if !(*(*connp).out_tx).response_status.is_null() {
                bstr_free((*(*connp).out_tx).response_status);
                (*(*connp).out_tx).response_status = 0 as *mut bstr
            }
            if !(*(*connp).out_tx).response_message.is_null() {
                bstr_free((*(*connp).out_tx).response_message);
                (*(*connp).out_tx).response_message = 0 as *mut bstr
            }
            // Process response line.
            let mut chomp_result: libc::c_int = htp_chomp(data, &mut len);
            // If the response line is invalid, determine if it _looks_ like
            // a response line. If it does not look like a line, process the
            // data as a response body because that is what browsers do.
            if htp_treat_response_line_as_body(data, len) != 0 {
                (*(*connp).out_tx).response_content_encoding_processing = HTP_COMPRESSION_NONE;
                (*connp).out_current_consume_offset = (*connp).out_current_read_offset;
                let mut rc: htp_status_t = htp_tx_res_process_body_data_ex(
                    (*connp).out_tx,
                    data as *const libc::c_void,
                    len.wrapping_add(chomp_result as libc::c_ulong),
                );
                if rc != 1 as libc::c_int {
                    return rc;
                }
                // Continue to process response body. Because we don't have
                // any headers to parse, we assume the body continues until
                // the end of the stream.
                // Have we seen the entire response body?
                if (*connp).out_current_len <= (*connp).out_current_read_offset {
                    (*(*connp).out_tx).response_transfer_coding = HTP_CODING_IDENTITY;
                    (*(*connp).out_tx).response_progress = HTP_RESPONSE_BODY;
                    (*connp).out_body_data_left = -(1 as libc::c_int) as int64_t;
                    (*connp).out_state = Some(
                        htp_connp_RES_FINALIZE
                            as unsafe extern "C" fn(
                                _: *mut crate::src::htp_connection_parser::htp_connp_t,
                            ) -> htp_status_t,
                    )
                }
                return 1 as libc::c_int;
            }
            (*(*connp).out_tx).response_line = bstr_dup_mem(data as *const libc::c_void, len);
            if (*(*connp).out_tx).response_line.is_null() {
                return -(1 as libc::c_int);
            }
            if (*(*connp).cfg)
                .parse_response_line
                .expect("non-null function pointer")(connp)
                != 1 as libc::c_int
            {
                return -(1 as libc::c_int);
            }
            let mut rc_0: htp_status_t = htp_tx_state_response_line((*connp).out_tx);
            if rc_0 != 1 as libc::c_int {
                return rc_0;
            }
            htp_connp_res_clear_buffer(connp);
            // Move on to the next phase.
            (*connp).out_state = Some(
                htp_connp_RES_HEADERS
                    as unsafe extern "C" fn(
                        _: *mut crate::src::htp_connection_parser::htp_connp_t,
                    ) -> htp_status_t,
            );
            (*(*connp).out_tx).response_progress = HTP_RESPONSE_HEADERS;
            return 1 as libc::c_int;
        }
    }
}

/* *
 * Returns the number of bytes consumed from the most recent outbound data chunk. Normally, an invocation
 * of htp_connp_res_data() will consume all data from the supplied buffer, but there are circumstances
 * where only partial consumption is possible. In such cases HTP_STREAM_DATA_OTHER will be returned.
 * Consumed bytes are no longer necessary, but the remainder of the buffer will be need to be saved
 * for later.
 *
 * @param[in] connp
 * @return The number of bytes consumed from the last data chunk sent for outbound processing.
 */
#[no_mangle]
pub unsafe extern "C" fn htp_connp_res_data_consumed(
    mut connp: *mut crate::src::htp_connection_parser::htp_connp_t,
) -> size_t {
    return (*connp).out_current_read_offset as size_t;
}
#[no_mangle]
pub unsafe extern "C" fn htp_connp_RES_FINALIZE(
    mut connp: *mut crate::src::htp_connection_parser::htp_connp_t,
) -> htp_status_t {
    if (*connp).out_status as libc::c_uint != HTP_STREAM_CLOSED as libc::c_int as libc::c_uint {
        if (*connp).out_current_read_offset >= (*connp).out_current_len {
            (*connp).out_next_byte = -(1 as libc::c_int)
        } else {
            (*connp).out_next_byte = *(*connp)
                .out_current_data
                .offset((*connp).out_current_read_offset as isize)
                as libc::c_int
        }
        if (*connp).out_next_byte == -(1 as libc::c_int) {
            return htp_tx_state_response_complete_ex((*connp).out_tx, 0 as libc::c_int);
        }
        if (*connp).out_next_byte != '\n' as i32
            || (*connp).out_current_consume_offset >= (*connp).out_current_read_offset
        {
            loop {
                //;i < max_read; i++) {
                if (*connp).out_current_read_offset < (*connp).out_current_len {
                    (*connp).out_next_byte = *(*connp)
                        .out_current_data
                        .offset((*connp).out_current_read_offset as isize)
                        as libc::c_int;
                    (*connp).out_current_read_offset += 1;
                    (*connp).out_stream_offset += 1
                } else {
                    return 5 as libc::c_int;
                }
                // Have we reached the end of the line? For some reason
                // we can't test after IN_COPY_BYTE_OR_RETURN */
                if (*connp).out_next_byte == '\n' as i32 {
                    break;
                }
            }
        }
    }
    let mut bytes_left: size_t = 0;
    let mut data: *mut libc::c_uchar = 0 as *mut libc::c_uchar;
    if htp_connp_res_consolidate_data(connp, &mut data, &mut bytes_left) != 1 as libc::c_int {
        return -(1 as libc::c_int);
    }
    if bytes_left == 0 as libc::c_int as libc::c_ulong {
        //closing
        return htp_tx_state_response_complete_ex((*connp).out_tx, 0 as libc::c_int);
    }
    if htp_treat_response_line_as_body(data, bytes_left) != 0 {
        // Interpret remaining bytes as body data
        htp_log(
            connp,
            b"htp_response.c\x00" as *const u8 as *const libc::c_char,
            1104 as libc::c_int,
            HTP_LOG_WARNING,
            0 as libc::c_int,
            b"Unexpected response body\x00" as *const u8 as *const libc::c_char,
        );
        let mut rc: htp_status_t = htp_tx_res_process_body_data_ex(
            (*connp).out_tx,
            data as *const libc::c_void,
            bytes_left,
        );
        htp_connp_res_clear_buffer(connp);
        return rc;
    }
    //unread last end of line so that RES_LINE works
    if (*connp).out_current_read_offset < bytes_left as int64_t {
        (*connp).out_current_read_offset = 0 as libc::c_int as int64_t
    } else {
        (*connp).out_current_read_offset = ((*connp).out_current_read_offset as libc::c_ulong)
            .wrapping_sub(bytes_left) as int64_t
            as int64_t
    }
    if (*connp).out_current_read_offset < (*connp).out_current_consume_offset {
        (*connp).out_current_consume_offset = (*connp).out_current_read_offset
    }
    return htp_tx_state_response_complete_ex((*connp).out_tx, 0 as libc::c_int);
}

/* *
 * The response idle state will initialize response processing, as well as
 * finalize each transactions after we are done with it.
 *
 * @param[in] connp
 * @returns HTP_OK on state change, HTP_ERROR on error, or HTP_DATA when more data is needed.
 */
#[no_mangle]
pub unsafe extern "C" fn htp_connp_RES_IDLE(
    mut connp: *mut crate::src::htp_connection_parser::htp_connp_t,
) -> htp_status_t {
    // We want to start parsing the next response (and change
    // the state from IDLE) only if there's at least one
    // byte of data available. Otherwise we could be creating
    // new structures even if there's no more data on the
    // connection.
    if (*connp).out_current_read_offset >= (*connp).out_current_len {
        return 2 as libc::c_int;
    }
    // Parsing a new response
    // Find the next outgoing transaction
    // If there is none, we just create one so that responses without
    // request can still be processed.
    (*connp).out_tx = htp_list_array_get((*(*connp).conn).transactions, (*connp).out_next_tx_index)
        as *mut crate::src::htp_transaction::htp_tx_t;
    if (*connp).out_tx.is_null() {
        htp_log(
            connp,
            b"htp_response.c\x00" as *const u8 as *const libc::c_char,
            1145 as libc::c_int,
            HTP_LOG_ERROR,
            0 as libc::c_int,
            b"Unable to match response to request\x00" as *const u8 as *const libc::c_char,
        );
        // finalize dangling request waiting for next request or body
        if (*connp).in_state
            == Some(
                htp_connp_REQ_FINALIZE
                    as unsafe extern "C" fn(
                        _: *mut crate::src::htp_connection_parser::htp_connp_t,
                    ) -> htp_status_t,
            )
        {
            htp_tx_state_request_complete((*connp).in_tx);
        }
        (*connp).out_tx = htp_connp_tx_create(connp);
        if (*connp).out_tx.is_null() {
            return -(1 as libc::c_int);
        }
        (*(*connp).out_tx).parsed_uri = htp_uri_alloc();
        if (*(*connp).out_tx).parsed_uri.is_null() {
            return -(1 as libc::c_int);
        }
        (*(*(*connp).out_tx).parsed_uri).path =
            bstr_dup_c(b"/libhtp::request_uri_not_seen\x00" as *const u8 as *const libc::c_char);
        if (*(*(*connp).out_tx).parsed_uri).path.is_null() {
            return -(1 as libc::c_int);
        }
        (*(*connp).out_tx).request_uri =
            bstr_dup_c(b"/libhtp::request_uri_not_seen\x00" as *const u8 as *const libc::c_char);
        if (*(*connp).out_tx).request_uri.is_null() {
            return -(1 as libc::c_int);
        }
        (*connp).in_state = Some(
            htp_connp_REQ_FINALIZE
                as unsafe extern "C" fn(
                    _: *mut crate::src::htp_connection_parser::htp_connp_t,
                ) -> htp_status_t,
        );
        // We've used one transaction
        (*connp).out_next_tx_index = (*connp).out_next_tx_index.wrapping_add(1)
    } else {
        // We've used one transaction
        (*connp).out_next_tx_index = (*connp).out_next_tx_index.wrapping_add(1);
        // TODO Detect state mismatch
        (*connp).out_content_length = -(1 as libc::c_int) as int64_t;
        (*connp).out_body_data_left = -(1 as libc::c_int) as int64_t
    }
    let mut rc: htp_status_t = htp_tx_state_response_start((*connp).out_tx);
    if rc != 1 as libc::c_int {
        return rc;
    }
    return 1 as libc::c_int;
}

/* *
 * Process a chunk of outbound (server or response) data.
 *
 * @param[in] connp
 * @param[in] timestamp Optional.
 * @param[in] data
 * @param[in] len
 * @return HTP_OK on state change, HTP_ERROR on error, or HTP_DATA when more data is needed
 */
#[no_mangle]
pub unsafe extern "C" fn htp_connp_res_data(
    mut connp: *mut crate::src::htp_connection_parser::htp_connp_t,
    mut timestamp: *const htp_time_t,
    mut data: *const libc::c_void,
    mut len: size_t,
) -> libc::c_int {
    // Return if the connection is in stop state
    if (*connp).out_status as libc::c_uint == HTP_STREAM_STOP as libc::c_int as libc::c_uint {
        htp_log(
            connp,
            b"htp_response.c\x00" as *const u8 as *const libc::c_char,
            1197 as libc::c_int,
            HTP_LOG_INFO,
            0 as libc::c_int,
            b"Outbound parser is in HTP_STREAM_STOP\x00" as *const u8 as *const libc::c_char,
        );
        return HTP_STREAM_STOP as libc::c_int;
    }
    // Return if the connection has had a fatal error
    if (*connp).out_status as libc::c_uint == HTP_STREAM_ERROR as libc::c_int as libc::c_uint {
        htp_log(
            connp,
            b"htp_response.c\x00" as *const u8 as *const libc::c_char,
            1204 as libc::c_int,
            HTP_LOG_ERROR,
            0 as libc::c_int,
            b"Outbound parser is in HTP_STREAM_ERROR\x00" as *const u8 as *const libc::c_char,
        );
        return HTP_STREAM_ERROR as libc::c_int;
    }
    // Sanity check: we must have a transaction pointer if the state is not IDLE (no outbound transaction)
    if (*connp).out_tx.is_null()
        && (*connp).out_state
            != Some(
                htp_connp_RES_IDLE
                    as unsafe extern "C" fn(
                        _: *mut crate::src::htp_connection_parser::htp_connp_t,
                    ) -> htp_status_t,
            )
    {
        (*connp).out_status = HTP_STREAM_ERROR;
        htp_log(
            connp,
            b"htp_response.c\x00" as *const u8 as *const libc::c_char,
            1217 as libc::c_int,
            HTP_LOG_ERROR,
            0 as libc::c_int,
            b"Missing outbound transaction data\x00" as *const u8 as *const libc::c_char,
        );
        return HTP_STREAM_ERROR as libc::c_int;
    }
    // If the length of the supplied data chunk is zero, proceed
    // only if the stream has been closed. We do not allow zero-sized
    // chunks in the API, but we use it internally to force the parsers
    // to finalize parsing.
    if (data == 0 as *mut libc::c_void || len == 0 as libc::c_int as libc::c_ulong)
        && (*connp).out_status as libc::c_uint != HTP_STREAM_CLOSED as libc::c_int as libc::c_uint
    {
        htp_log(
            connp,
            b"htp_response.c\x00" as *const u8 as *const libc::c_char,
            1227 as libc::c_int,
            HTP_LOG_ERROR,
            0 as libc::c_int,
            b"Zero-length data chunks are not allowed\x00" as *const u8 as *const libc::c_char,
        );
        return HTP_STREAM_CLOSED as libc::c_int;
    }
    // Remember the timestamp of the current response data chunk
    if !timestamp.is_null() {
        memcpy(
            &mut (*connp).out_timestamp as *mut htp_time_t as *mut libc::c_void,
            timestamp as *const libc::c_void,
            ::std::mem::size_of::<htp_time_t>() as libc::c_ulong,
        );
    }
    // Store the current chunk information
    (*connp).out_current_data = data as *mut libc::c_uchar;
    (*connp).out_current_len = len as int64_t;
    (*connp).out_current_read_offset = 0 as libc::c_int as int64_t;
    (*connp).out_current_consume_offset = 0 as libc::c_int as int64_t;
    (*connp).out_current_receiver_offset = 0 as libc::c_int as int64_t;
    htp_conn_track_outbound_data((*connp).conn, len, timestamp);
    // Return without processing any data if the stream is in tunneling
    // mode (which it would be after an initial CONNECT transaction.
    if (*connp).out_status as libc::c_uint == HTP_STREAM_TUNNEL as libc::c_int as libc::c_uint {
        return HTP_STREAM_TUNNEL as libc::c_int;
    }
    loop
    // Invoke a processor, in a loop, until an error
    // occurs or until we run out of data. Many processors
    // will process a request, each pointing to the next
    // processor that needs to run.
    // Return if there's been an error
    // or if we've run out of data. We are relying
    // on processors to add error messages, so we'll
    // keep quiet here.
    {
        let mut rc: htp_status_t = (*connp).out_state.expect("non-null function pointer")(connp);
        if rc == 1 as libc::c_int {
            if (*connp).out_status as libc::c_uint
                == HTP_STREAM_TUNNEL as libc::c_int as libc::c_uint
            {
                return HTP_STREAM_TUNNEL as libc::c_int;
            }
            rc = htp_res_handle_state_change(connp)
        }
        if rc != 1 as libc::c_int {
            // Do we need more data?
            if rc == 2 as libc::c_int || rc == 5 as libc::c_int {
                htp_connp_res_receiver_send_data(connp, 0 as libc::c_int);
                if rc == 5 as libc::c_int {
                    if htp_connp_res_buffer(connp) != 1 as libc::c_int {
                        (*connp).out_status = HTP_STREAM_ERROR;
                        return HTP_STREAM_ERROR as libc::c_int;
                    }
                }
                (*connp).out_status = HTP_STREAM_DATA;
                return HTP_STREAM_DATA as libc::c_int;
            }
            // Check for stop
            if rc == 4 as libc::c_int {
                (*connp).out_status = HTP_STREAM_STOP;
                return HTP_STREAM_STOP as libc::c_int;
            }
            // Check for suspended parsing
            if rc == 3 as libc::c_int {
                // We might have actually consumed the entire data chunk?
                if (*connp).out_current_read_offset >= (*connp).out_current_len {
                    (*connp).out_status = HTP_STREAM_DATA;
                    // Do not send STREAM_DATE_DATA_OTHER if we've
                    // consumed the entire chunk
                    return HTP_STREAM_DATA as libc::c_int;
                } else {
                    (*connp).out_status = HTP_STREAM_DATA_OTHER;
                    // Partial chunk consumption
                    return HTP_STREAM_DATA_OTHER as libc::c_int;
                }
            }
            // Permanent stream error.
            (*connp).out_status = HTP_STREAM_ERROR;
            return HTP_STREAM_ERROR as libc::c_int;
        }
    }
}
