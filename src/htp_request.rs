use crate::bstr::{bstr_len, bstr_ptr};
use crate::error::Result;
use crate::hook::DataHook;
use crate::htp_connection_parser::State;
use crate::htp_util::Flags;
use crate::{bstr, htp_connection_parser, htp_transaction, htp_util, Status};

extern "C" {
    #[no_mangle]
    fn malloc(_: libc::size_t) -> *mut core::ffi::c_void;
    #[no_mangle]
    fn realloc(_: *mut core::ffi::c_void, _: libc::size_t) -> *mut core::ffi::c_void;
    #[no_mangle]
    fn free(__ptr: *mut core::ffi::c_void);
    #[no_mangle]
    fn memcpy(
        _: *mut core::ffi::c_void,
        _: *const core::ffi::c_void,
        _: libc::size_t,
    ) -> *mut core::ffi::c_void;
}

/// HTTP methods.
#[repr(C)]
#[derive(Copy, Clone, PartialEq, Debug)]
pub enum htp_method_t {
    /// Used by default, until the method is determined (e.g., before
    /// the request line is processed.
    HTP_M_UNKNOWN,
    HTP_M_HEAD,
    HTP_M_GET,
    HTP_M_PUT,
    HTP_M_POST,
    HTP_M_DELETE,
    HTP_M_CONNECT,
    HTP_M_OPTIONS,
    HTP_M_TRACE,
    HTP_M_PATCH,
    HTP_M_PROPFIND,
    HTP_M_PROPPATCH,
    HTP_M_MKCOL,
    HTP_M_COPY,
    HTP_M_MOVE,
    HTP_M_LOCK,
    HTP_M_UNLOCK,
    HTP_M_VERSION_CONTROL,
    HTP_M_CHECKOUT,
    HTP_M_UNCHECKOUT,
    HTP_M_CHECKIN,
    HTP_M_UPDATE,
    HTP_M_LABEL,
    HTP_M_REPORT,
    HTP_M_MKWORKSPACE,
    HTP_M_MKACTIVITY,
    HTP_M_BASELINE_CONTROL,
    HTP_M_MERGE,
    HTP_M_INVALID,
    HTP_M_ERROR,
}

pub type htp_time_t = libc::timeval;

/// Sends outstanding connection data to the currently active data receiver hook.
///
/// Returns HTP_OK, or a value returned from a callback.
unsafe fn htp_connp_req_receiver_send_data(
    connp: &mut htp_connection_parser::htp_connp_t,
    is_last: bool,
) -> Result<()> {
    let mut data = htp_transaction::htp_tx_data_t::new(
        (*connp).in_tx_mut_ptr(),
        (*connp)
            .in_current_data
            .offset((*connp).in_current_receiver_offset as isize),
        ((*connp).in_current_read_offset - (*connp).in_current_receiver_offset) as usize,
        is_last,
    );
    if let Some(hook) = &(*connp).in_data_receiver_hook {
        hook.run_all(&mut data)?;
    } else {
        return Ok(());
    };
    (*connp).in_current_receiver_offset = (*connp).in_current_read_offset;
    Ok(())
}

/// Configures the data receiver hook. If there is a previous hook, it will be finalized and cleared.
///
/// Returns HTP_OK, or a value returned from a callback.
unsafe fn htp_connp_req_receiver_set(
    connp: &mut htp_connection_parser::htp_connp_t,
    data_receiver_hook: Option<DataHook>,
) -> Result<()> {
    // Ignore result.
    let _ = htp_connp_req_receiver_finalize_clear(connp);
    (*connp).in_data_receiver_hook = data_receiver_hook;
    (*connp).in_current_receiver_offset = (*connp).in_current_read_offset;
    Ok(())
}

/// Finalizes an existing data receiver hook by sending any outstanding data to it. The
/// hook is then removed so that it receives no more data.
///
/// Returns HTP_OK, or a value returned from a callback.
pub unsafe fn htp_connp_req_receiver_finalize_clear(
    connp: &mut htp_connection_parser::htp_connp_t,
) -> Result<()> {
    if (*connp).in_data_receiver_hook.is_none() {
        return Ok(());
    }
    let rc = htp_connp_req_receiver_send_data(connp, true);
    (*connp).in_data_receiver_hook = None;
    rc
}

/// Handles request parser state changes. At the moment, this function is used only
/// to configure data receivers, which are sent raw connection data.
///
/// Returns HTP_OK, or a value returned from a callback.
unsafe fn htp_req_handle_state_change(
    connp: &mut htp_connection_parser::htp_connp_t,
) -> Result<()> {
    if (*connp).in_state_previous == (*connp).in_state {
        return Ok(());
    }
    if (*connp).in_state == State::HEADERS {
        let header_fn = Some(
            (*(*connp).in_tx_mut_ok()?.cfg)
                .hook_request_header_data
                .clone(),
        );
        let trailer_fn = Some(
            (*(*connp).in_tx_mut_ok()?.cfg)
                .hook_request_trailer_data
                .clone(),
        );

        match (*connp).in_tx_mut_ok()?.request_progress as u32 {
            2 => htp_connp_req_receiver_set(connp, header_fn),
            4 => htp_connp_req_receiver_set(connp, trailer_fn),
            _ => Ok(()),
        }?;
    }
    // Initially, I had the finalization of raw data sending here, but that
    // caused the last REQUEST_HEADER_DATA hook to be invoked after the
    // REQUEST_HEADERS hook -- which I thought made no sense. For that reason,
    // the finalization is now initiated from the request header processing code,
    // which is less elegant but provides a better user experience. Having some
    // (or all) hooks to be invoked on state change might work better.
    (*connp).in_state_previous = (*connp).in_state;
    Ok(())
}

/// If there is any data left in the inbound data chunk, this function will preserve
/// it for later consumption. The maximum amount accepted for buffering is controlled
/// by htp_config_t::field_limit_hard.
///
/// Returns HTP_OK, or HTP_ERROR on fatal failure.
unsafe fn htp_connp_req_buffer(connp: &mut htp_connection_parser::htp_connp_t) -> Result<()> {
    if (*connp).in_current_data.is_null() {
        return Ok(());
    }
    let data: *mut u8 = (*connp)
        .in_current_data
        .offset((*connp).in_current_consume_offset as isize);
    let len: usize =
        ((*connp).in_current_read_offset - (*connp).in_current_consume_offset) as usize;
    if len == 0 {
        return Ok(());
    }
    // Check the hard (buffering) limit.
    let mut newlen: usize = (*connp).in_buf_size.wrapping_add(len);
    // When calculating the size of the buffer, take into account the
    // space we're using for the request header buffer.
    if !(*connp).in_header.is_null() {
        newlen = newlen.wrapping_add(bstr_len((*connp).in_header))
    }
    if newlen > (*(*connp).in_tx_mut_ok()?.cfg).field_limit_hard {
        htp_error!(
            connp as *mut htp_connection_parser::htp_connp_t,
            htp_log_code::REQUEST_FIELD_TOO_LONG,
            format!(
                "Request buffer over the limit: size {} limit {}.",
                newlen,
                (*(*connp).in_tx_mut_ok()?.cfg).field_limit_hard
            )
        );
        return Err(Status::ERROR);
    }
    // Copy the data remaining in the buffer.
    if (*connp).in_buf.is_null() {
        (*connp).in_buf = malloc(len) as *mut u8;
        if (*connp).in_buf.is_null() {
            return Err(Status::ERROR);
        }
        memcpy(
            (*connp).in_buf as *mut core::ffi::c_void,
            data as *const core::ffi::c_void,
            len,
        );
        (*connp).in_buf_size = len
    } else {
        let newsize: usize = (*connp).in_buf_size.wrapping_add(len);
        let newbuf: *mut u8 =
            realloc((*connp).in_buf as *mut core::ffi::c_void, newsize) as *mut u8;
        if newbuf.is_null() {
            return Err(Status::ERROR);
        }
        (*connp).in_buf = newbuf;
        memcpy(
            (*connp).in_buf.offset((*connp).in_buf_size as isize) as *mut core::ffi::c_void,
            data as *const core::ffi::c_void,
            len,
        );
        (*connp).in_buf_size = newsize
    }
    // Reset the consumer position.
    (*connp).in_current_consume_offset = (*connp).in_current_read_offset;
    Ok(())
}

/// Returns to the caller the memory region that should be processed next. This function
/// hides away the buffering process from the rest of the code, allowing it to work with
/// non-buffered data that's in the inbound chunk, or buffered data that's in our structures.
///
/// Returns HTP_OK
unsafe fn htp_connp_req_consolidate_data(
    connp: &mut htp_connection_parser::htp_connp_t,
    data: *mut *mut u8,
    len: *mut usize,
) -> Result<()> {
    if (*connp).in_buf.is_null() {
        // We do not have any data buffered; point to the current data chunk.
        *data = (*connp)
            .in_current_data
            .offset((*connp).in_current_consume_offset as isize);
        *len = ((*connp).in_current_read_offset - (*connp).in_current_consume_offset) as usize
    } else {
        // We already have some data in the buffer. Add the data from the current
        // chunk to it, and point to the consolidated buffer.
        htp_connp_req_buffer(connp)?;
        *data = (*connp).in_buf;
        *len = (*connp).in_buf_size
    }
    Ok(())
}

/// Clears buffered inbound data and resets the consumer position to the reader position.
unsafe fn htp_connp_req_clear_buffer(connp: &mut htp_connection_parser::htp_connp_t) {
    (*connp).in_current_consume_offset = (*connp).in_current_read_offset;
    if !(*connp).in_buf.is_null() {
        free((*connp).in_buf as *mut core::ffi::c_void);
        (*connp).in_buf = 0 as *mut u8;
        (*connp).in_buf_size = 0
    };
}

/// Performs a check for a CONNECT transaction to decide whether inbound
/// parsing needs to be suspended.
///
/// Returns HTP_OK if the request does not use CONNECT, HTP_DATA_OTHER if
///          inbound parsing needs to be suspended until we hear from the
///          other side
pub unsafe fn htp_connp_REQ_CONNECT_CHECK(
    connp: &mut htp_connection_parser::htp_connp_t,
) -> Result<()> {
    // If the request uses the CONNECT method, then there will
    // not be a request body, but first we need to wait to see the
    // response in order to determine if the tunneling request
    // was a success.
    if connp.in_tx_mut_ok()?.request_method_number == htp_method_t::HTP_M_CONNECT {
        (*connp).in_state = State::CONNECT_WAIT_RESPONSE;
        (*connp).in_status = htp_connection_parser::htp_stream_state_t::HTP_STREAM_DATA_OTHER;
        return Err(Status::DATA_OTHER);
    }
    // Continue to the next step to determine
    // the presence of request body
    (*connp).in_state = State::BODY_DETERMINE;
    Ok(())
}

/// Determines whether inbound parsing needs to continue or stop. In
/// case the data appears to be plain text HTTP, we try to continue.
///
/// Returns HTP_OK if the parser can resume parsing, HTP_DATA_BUFFER if
///         we need more data.
pub unsafe fn htp_connp_REQ_CONNECT_PROBE_DATA(
    connp: &mut htp_connection_parser::htp_connp_t,
) -> Result<()> {
    loop {
        //;i < max_read; i++) {
        if (*connp).in_current_read_offset >= (*connp).in_current_len {
            (*connp).in_next_byte = -1
        } else {
            (*connp).in_next_byte = *(*connp)
                .in_current_data
                .offset((*connp).in_current_read_offset as isize)
                as i32
        }
        // Have we reached the end of the line? For some reason
        // we can't test after IN_COPY_BYTE_OR_RETURN */
        if (*connp).in_next_byte == '\n' as i32 || (*connp).in_next_byte == 0 {
            break;
        }
        if (*connp).in_current_read_offset < (*connp).in_current_len {
            (*connp).in_next_byte = *(*connp)
                .in_current_data
                .offset((*connp).in_current_read_offset as isize)
                as i32;
            (*connp).in_current_read_offset += 1;
            (*connp).in_stream_offset += 1
        } else {
            return Err(Status::DATA_BUFFER);
        }
    }
    let mut data: *mut u8 = 0 as *mut u8;
    let mut len: usize = 0;
    htp_connp_req_consolidate_data(connp, &mut data, &mut len)?;
    let mut pos: usize = 0;
    let mut mstart: usize = 0;
    // skip past leading whitespace. IIS allows this
    while pos < len && htp_util::htp_is_space(*data.offset(pos as isize)) {
        pos = pos.wrapping_add(1)
    }
    if pos != 0 {
        mstart = pos
    }
    // The request method starts at the beginning of the
    // line and ends with the first whitespace character.
    while pos < len && !htp_util::htp_is_space(*data.offset(pos as isize)) {
        pos = pos.wrapping_add(1)
    }
    let mut method_type = htp_method_t::HTP_M_UNKNOWN;
    let method: *mut bstr::bstr_t = bstr::bstr_dup_mem(
        data.offset(mstart as isize) as *const core::ffi::c_void,
        pos.wrapping_sub(mstart),
    );
    if !method.is_null() {
        method_type = htp_util::htp_convert_bstr_to_method(&*method);
        bstr::bstr_free(method);
    }
    if method_type != htp_method_t::HTP_M_UNKNOWN {
        return (*connp).state_request_complete().into();
    } else {
        (*connp).in_status = htp_connection_parser::htp_stream_state_t::HTP_STREAM_TUNNEL;
        (*connp).out_status = htp_connection_parser::htp_stream_state_t::HTP_STREAM_TUNNEL
    }
    // not calling htp_connp_req_clear_buffer, we're not consuming the data
    Ok(())
}

/// Determines whether inbound parsing, which was suspended after
/// encountering a CONNECT transaction, can proceed (after receiving
/// the response).
///
/// Returns HTP_OK if the parser can resume parsing, HTP_DATA_OTHER if
///         it needs to continue waiting.
pub unsafe fn htp_connp_REQ_CONNECT_WAIT_RESPONSE(
    connp: &mut htp_connection_parser::htp_connp_t,
) -> Result<()> {
    // Check that we saw the response line of the current inbound transaction.
    if connp.in_tx_mut_ok()?.response_progress
        <= htp_transaction::htp_tx_res_progress_t::HTP_RESPONSE_LINE
    {
        return Err(Status::DATA_OTHER);
    }
    // A 2xx response means a tunnel was established. Anything
    // else means we continue to follow the HTTP stream.
    if connp.in_tx_mut_ok()?.response_status_number >= 200
        && connp.in_tx_mut_ok()?.response_status_number <= 299
    {
        // TODO Check that the server did not accept a connection to itself.
        // The requested tunnel was established: we are going
        // to probe the remaining data on this stream to see
        // if we need to ignore it or parse it
        (*connp).in_state = State::CONNECT_PROBE_DATA;
    } else {
        // No tunnel; continue to the next transaction
        (*connp).in_state = State::FINALIZE
    }
    Ok(())
}

/// Consumes bytes until the end of the current line.
///
/// Returns HTP_OK on state change, HTP_ERROR on error, or HTP_DATA when more data is needed.
pub unsafe fn htp_connp_REQ_BODY_CHUNKED_DATA_END(
    connp: &mut htp_connection_parser::htp_connp_t,
) -> Result<()> {
    loop
    // TODO We shouldn't really see anything apart from CR and LF,
    //      so we should warn about anything else.
    {
        if (*connp).in_current_read_offset < (*connp).in_current_len {
            (*connp).in_next_byte = *(*connp)
                .in_current_data
                .offset((*connp).in_current_read_offset as isize)
                as i32;
            (*connp).in_current_read_offset += 1;
            (*connp).in_current_consume_offset += 1;
            (*connp).in_stream_offset += 1
        } else {
            return Err(Status::DATA);
        }
        connp.in_tx_mut_ok()?.request_message_len += 1;
        if (*connp).in_next_byte == '\n' as i32 {
            (*connp).in_state = State::BODY_CHUNKED_LENGTH;
            return Ok(());
        }
    }
}

/// Processes a chunk of data.
///
/// Returns HTP_OK on state change, HTP_ERROR on error, or HTP_DATA when more data is needed.
pub unsafe fn htp_connp_REQ_BODY_CHUNKED_DATA(
    connp: &mut htp_connection_parser::htp_connp_t,
) -> Result<()> {
    // Determine how many bytes we can consume.
    let mut bytes_to_consume: usize = 0;
    if (*connp).in_current_len - (*connp).in_current_read_offset >= (*connp).in_chunked_length {
        // Entire chunk available in the buffer; read all of it.
        bytes_to_consume = (*connp).in_chunked_length as usize
    } else {
        // Partial chunk available in the buffer; read as much as we can.
        bytes_to_consume = ((*connp).in_current_len - (*connp).in_current_read_offset) as usize
    }
    // If the input buffer is empty, ask for more data.
    if bytes_to_consume == 0 {
        return Err(Status::DATA);
    }
    // Consume the data.
    (*connp).req_process_body_data_ex(
        (*connp)
            .in_current_data
            .offset((*connp).in_current_read_offset as isize) as *const core::ffi::c_void,
        bytes_to_consume,
    )?;
    // Adjust counters.
    (*connp).in_current_read_offset =
        ((*connp).in_current_read_offset as u64).wrapping_add(bytes_to_consume as u64) as i64;
    (*connp).in_current_consume_offset =
        ((*connp).in_current_consume_offset as u64).wrapping_add(bytes_to_consume as u64) as i64;
    (*connp).in_stream_offset =
        ((*connp).in_stream_offset as u64).wrapping_add(bytes_to_consume as u64) as i64;
    connp.in_tx_mut_ok()?.request_message_len = (connp.in_tx_mut_ok()?.request_message_len as u64)
        .wrapping_add(bytes_to_consume as u64)
        as i64;
    (*connp).in_chunked_length =
        ((*connp).in_chunked_length as u64).wrapping_sub(bytes_to_consume as u64) as i64;
    if (*connp).in_chunked_length == 0 {
        // End of the chunk.
        (*connp).in_state = State::BODY_CHUNKED_DATA_END;
        return Ok(());
    }
    // Ask for more data.
    Err(Status::DATA)
}

/// Extracts chunk length.
///
/// Returns HTP_OK on state change, HTP_ERROR on error, or HTP_DATA when more data is needed.
pub unsafe fn htp_connp_REQ_BODY_CHUNKED_LENGTH(
    connp: &mut htp_connection_parser::htp_connp_t,
) -> Result<()> {
    loop {
        if (*connp).in_current_read_offset < (*connp).in_current_len {
            (*connp).in_next_byte = *(*connp)
                .in_current_data
                .offset((*connp).in_current_read_offset as isize)
                as i32;
            (*connp).in_current_read_offset += 1;
            (*connp).in_stream_offset += 1
        } else {
            return Err(Status::DATA_BUFFER);
        }
        // Have we reached the end of the line?
        if (*connp).in_next_byte == '\n' as i32 {
            let mut data: *mut u8 = 0 as *mut u8;
            let mut len: usize = 0;
            htp_connp_req_consolidate_data(connp, &mut data, &mut len)?;
            connp.in_tx_mut_ok()?.request_message_len =
                (connp.in_tx_mut_ok()?.request_message_len as u64).wrapping_add(len as u64) as i64;
            let buf: &mut [u8] = std::slice::from_raw_parts_mut(data, len);
            if let Ok(Some(chunked_len)) = htp_util::htp_parse_chunked_length(buf) {
                (*connp).in_chunked_length = chunked_len as i64;
            } else {
                (*connp).in_chunked_length = -1;
            }
            htp_connp_req_clear_buffer(connp);
            // Handle chunk length.
            if (*connp).in_chunked_length > 0 {
                // More data available.
                (*connp).in_state = State::BODY_CHUNKED_DATA
            } else if (*connp).in_chunked_length == 0 {
                // End of data.
                (*connp).in_state = State::HEADERS;
                connp.in_tx_mut_ok()?.request_progress =
                    htp_transaction::htp_tx_req_progress_t::HTP_REQUEST_TRAILER
            } else {
                // Invalid chunk length.
                htp_error!(
                    connp as *mut htp_connection_parser::htp_connp_t,
                    htp_log_code::INVALID_REQUEST_CHUNK_LEN,
                    "Request chunk encoding: Invalid chunk length"
                );
                return Err(Status::ERROR);
            }
            return Ok(());
        }
    }
}

/// Processes identity request body.
///
/// Returns HTP_OK on state change, HTP_ERROR on error, or HTP_DATA when more data is needed.
pub unsafe fn htp_connp_REQ_BODY_IDENTITY(
    connp: &mut htp_connection_parser::htp_connp_t,
) -> Result<()> {
    // Determine how many bytes we can consume.
    let mut bytes_to_consume: usize = 0;
    if (*connp).in_current_len - (*connp).in_current_read_offset >= (*connp).in_body_data_left {
        bytes_to_consume = (*connp).in_body_data_left as usize
    } else {
        bytes_to_consume = ((*connp).in_current_len - (*connp).in_current_read_offset) as usize
    }
    // If the input buffer is empty, ask for more data.
    if bytes_to_consume == 0 {
        return Err(Status::DATA);
    }
    // Consume data.
    (*connp).req_process_body_data_ex(
        (*connp)
            .in_current_data
            .offset((*connp).in_current_read_offset as isize) as *const core::ffi::c_void,
        bytes_to_consume,
    )?;
    // Adjust counters.
    (*connp).in_current_read_offset =
        ((*connp).in_current_read_offset as u64).wrapping_add(bytes_to_consume as u64) as i64;
    (*connp).in_current_consume_offset =
        ((*connp).in_current_consume_offset as u64).wrapping_add(bytes_to_consume as u64) as i64;
    (*connp).in_stream_offset =
        ((*connp).in_stream_offset as u64).wrapping_add(bytes_to_consume as u64) as i64;
    connp.in_tx_mut_ok()?.request_message_len = (connp.in_tx_mut_ok()?.request_message_len as u64)
        .wrapping_add(bytes_to_consume as u64)
        as i64;
    (*connp).in_body_data_left =
        ((*connp).in_body_data_left as u64).wrapping_sub(bytes_to_consume as u64) as i64;
    if (*connp).in_body_data_left == 0 {
        // End of request body.
        (*connp).in_state = State::FINALIZE;
        return Ok(());
    }
    // Ask for more data.
    Err(Status::DATA)
}

/// Determines presence (and encoding) of a request body.
///
/// Returns HTP_OK on state change, HTP_ERROR on error, or HTP_DATA when more data is needed.
pub unsafe fn htp_connp_REQ_BODY_DETERMINE(
    connp: &mut htp_connection_parser::htp_connp_t,
) -> Result<()> {
    // Determine the next state based on the presence of the request
    // body, and the coding used.
    match connp.in_tx_mut_ok()?.request_transfer_coding as u32 {
        3 => {
            (*connp).in_state = State::BODY_CHUNKED_LENGTH;
            connp.in_tx_mut_ok()?.request_progress =
                htp_transaction::htp_tx_req_progress_t::HTP_REQUEST_BODY
        }
        2 => {
            (*connp).in_content_length = connp.in_tx_mut_ok()?.request_content_length;
            (*connp).in_body_data_left = (*connp).in_content_length;
            if (*connp).in_content_length != 0 {
                (*connp).in_state = State::BODY_IDENTITY;
                connp.in_tx_mut_ok()?.request_progress =
                    htp_transaction::htp_tx_req_progress_t::HTP_REQUEST_BODY
            } else {
                (*connp.in_tx_mut_ok()?.connp).in_state = State::FINALIZE
            }
        }
        1 => {
            // This request does not have a body, which
            // means that we're done with it
            (*connp).in_state = State::FINALIZE
        }
        _ => {
            // Should not be here
            return Err(Status::ERROR);
        }
    }
    Ok(())
}

/// Parses request headers.
///
/// Returns HTP_OK on state change, HTP_ERROR on error, or HTP_DATA when more data is needed.
pub unsafe fn htp_connp_REQ_HEADERS(connp: &mut htp_connection_parser::htp_connp_t) -> Result<()> {
    loop {
        if (*connp).in_status == htp_connection_parser::htp_stream_state_t::HTP_STREAM_CLOSED {
            // Parse previous header, if any.
            if !(*connp).in_header.is_null() {
                (*connp).process_request_header(
                    bstr_ptr((*connp).in_header),
                    bstr_len((*connp).in_header),
                )?;
                bstr::bstr_free((*connp).in_header);
                (*connp).in_header = 0 as *mut bstr::bstr_t
            }
            htp_connp_req_clear_buffer(connp);
            connp.in_tx_mut_ok()?.request_progress =
                htp_transaction::htp_tx_req_progress_t::HTP_REQUEST_TRAILER;
            // We've seen all the request headers.
            return (*connp).state_request_headers().into();
        }
        if (*connp).in_current_read_offset < (*connp).in_current_len {
            (*connp).in_next_byte = *(*connp)
                .in_current_data
                .offset((*connp).in_current_read_offset as isize)
                as i32;
            (*connp).in_current_read_offset += 1;
            (*connp).in_stream_offset += 1
        } else {
            return Err(Status::DATA_BUFFER);
        }
        // Have we reached the end of the line?
        if (*connp).in_next_byte == '\n' as i32 {
            let mut data: *mut u8 = 0 as *mut u8;
            let mut len: usize = 0;
            htp_connp_req_consolidate_data(connp, &mut data, &mut len)?;
            // Should we terminate headers?
            if !data.is_null()
                && htp_util::htp_connp_is_line_terminator(
                    (*(*connp).cfg).server_personality,
                    std::slice::from_raw_parts(data, len),
                    false,
                )
            {
                // Parse previous header, if any.
                if !(*connp).in_header.is_null() {
                    (*connp).process_request_header(
                        bstr_ptr((*connp).in_header),
                        bstr_len((*connp).in_header),
                    )?;
                    bstr::bstr_free((*connp).in_header);
                    (*connp).in_header = 0 as *mut bstr::bstr_t
                }
                htp_connp_req_clear_buffer(connp);
                // We've seen all the request headers.
                return (*connp).state_request_headers().into();
            }
            let s = std::slice::from_raw_parts(data as *const u8, len);
            let s = htp_util::htp_chomp(&s);
            len = s.len();
            // Check for header folding.
            if !data.is_null()
                && !htp_util::htp_connp_is_line_folded(std::slice::from_raw_parts(data, len))
            {
                // New header line.
                // Parse previous header, if any.
                if !(*connp).in_header.is_null() {
                    (*connp).process_request_header(
                        bstr_ptr((*connp).in_header),
                        bstr_len((*connp).in_header),
                    )?;
                    bstr::bstr_free((*connp).in_header);
                    (*connp).in_header = 0 as *mut bstr::bstr_t
                }
                if (*connp).in_current_read_offset >= (*connp).in_current_len {
                    (*connp).in_next_byte = -1
                } else {
                    (*connp).in_next_byte = *(*connp)
                        .in_current_data
                        .offset((*connp).in_current_read_offset as isize)
                        as i32;
                }
                if (*connp).in_next_byte != -1
                    && !htp_util::htp_is_folding_char((*connp).in_next_byte as u8)
                {
                    // Because we know this header is not folded, we can process the buffer straight away.
                    (*connp).process_request_header(data, len)?;
                } else {
                    // Keep the partial header data for parsing later.
                    (*connp).in_header = bstr::bstr_dup_mem(data as *const core::ffi::c_void, len);
                    if (*connp).in_header.is_null() {
                        return Err(Status::ERROR);
                    }
                }
            } else if (*connp).in_header.is_null() {
                // Folding; check that there's a previous header line to add to.
                // Invalid folding.
                // Warn only once per transaction.
                if !connp
                    .in_tx_mut_ok()?
                    .flags
                    .contains(Flags::HTP_INVALID_FOLDING)
                {
                    connp.in_tx_mut_ok()?.flags |= Flags::HTP_INVALID_FOLDING;
                    htp_warn!(
                        connp as *mut htp_connection_parser::htp_connp_t,
                        htp_log_code::INVALID_REQUEST_FIELD_FOLDING,
                        "Invalid request field folding"
                    );
                }
                // Keep the header data for parsing later.
                (*connp).in_header = bstr::bstr_dup_mem(data as *const core::ffi::c_void, len);
                if (*connp).in_header.is_null() {
                    return Err(Status::ERROR);
                }
            } else {
                // Add to the existing header.
                let new_in_header: *mut bstr::bstr_t =
                    bstr::bstr_add_mem((*connp).in_header, data as *const core::ffi::c_void, len);
                if new_in_header.is_null() {
                    return Err(Status::ERROR);
                }
                (*connp).in_header = new_in_header
            }
            htp_connp_req_clear_buffer(connp);
        }
    }
}

/// Determines request protocol.
///
/// Returns HTP_OK on state change, HTP_ERROR on error, or HTP_DATA when more data is needed.
pub unsafe fn htp_connp_REQ_PROTOCOL(connp: &mut htp_connection_parser::htp_connp_t) -> Result<()> {
    // Is this a short-style HTTP/0.9 request? If it is,
    // we will not want to parse request headers.
    if connp.in_tx_mut_ok()?.is_protocol_0_9 == 0 {
        // Switch to request header parsing.
        (*connp).in_state = State::HEADERS;
        connp.in_tx_mut_ok()?.request_progress =
            htp_transaction::htp_tx_req_progress_t::HTP_REQUEST_HEADERS
    } else {
        // Let's check if the protocol was simply missing
        let mut pos: i64 = (*connp).in_current_read_offset;
        let mut afterspaces: i32 = 0;
        // Probe if data looks like a header line
        while pos < (*connp).in_current_len {
            if *(*connp).in_current_data.offset(pos as isize) == ':' as u8 {
                htp_warn!(
                    connp as *mut htp_connection_parser::htp_connp_t,
                    htp_log_code::REQUEST_LINE_NO_PROTOCOL,
                    "Request line: missing protocol"
                );
                connp.in_tx_mut_ok()?.is_protocol_0_9 = 0;
                // Switch to request header parsing.
                (*connp).in_state = State::HEADERS;
                connp.in_tx_mut_ok()?.request_progress =
                    htp_transaction::htp_tx_req_progress_t::HTP_REQUEST_HEADERS;
                return Ok(());
            } else {
                if htp_util::htp_is_lws(*(*connp).in_current_data.offset(pos as isize)) {
                    // Allows spaces after header name
                    afterspaces = 1
                } else if htp_util::htp_is_space(*(*connp).in_current_data.offset(pos as isize))
                    || afterspaces == 1
                {
                    break;
                }
                pos += 1
            }
        }
        // We're done with this request.
        (*connp).in_state = State::FINALIZE
    }
    Ok(())
}

/// Parse the request line.
///
/// Returns HTP_OK on succesful parse, HTP_ERROR on error.
pub unsafe fn htp_connp_REQ_LINE_complete(
    connp: &mut htp_connection_parser::htp_connp_t,
) -> Result<()> {
    let mut data: *mut u8 = 0 as *mut u8;
    let mut len: usize = 0;
    htp_connp_req_consolidate_data(connp, &mut data, &mut len)?;
    // Is this a line that should be ignored?
    if !data.is_null()
        && htp_util::htp_connp_is_line_ignorable(
            (*(*connp).cfg).server_personality,
            std::slice::from_raw_parts(data, len),
        )
    {
        // We have an empty/whitespace line, which we'll note, ignore and move on.
        connp.in_tx_mut_ok()?.request_ignored_lines =
            connp.in_tx_mut_ok()?.request_ignored_lines.wrapping_add(1);
        htp_connp_req_clear_buffer(connp);
        return Ok(());
    }
    // Process request line.
    let s = std::slice::from_raw_parts(data as *const u8, len);
    let s = htp_util::htp_chomp(&s);
    len = s.len();
    connp.in_tx_mut_ok()?.request_line = bstr::bstr_dup_mem(data as *const core::ffi::c_void, len);
    if connp.in_tx_mut_ok()?.request_line.is_null() {
        return Err(Status::ERROR);
    }
    if (*connp).parse_request_line().is_err() {
        return Err(Status::ERROR);
    }
    // Finalize request line parsing.
    if (*connp).state_request_line().is_err() {
        return Err(Status::ERROR);
    }
    htp_connp_req_clear_buffer(connp);
    Ok(())
}

/// Parses request line.
///
/// Returns HTP_OK on state change, HTP_ERROR on error, or HTP_DATA when more data is needed.
pub unsafe fn htp_connp_REQ_LINE(connp: &mut htp_connection_parser::htp_connp_t) -> Result<()> {
    loop {
        // Get one byte
        if (*connp).in_current_read_offset >= (*connp).in_current_len {
            (*connp).in_next_byte = -1
        } else {
            (*connp).in_next_byte = *(*connp)
                .in_current_data
                .offset((*connp).in_current_read_offset as isize)
                as i32
        }
        if (*connp).in_status == htp_connection_parser::htp_stream_state_t::HTP_STREAM_CLOSED
            && (*connp).in_next_byte == -1
        {
            return htp_connp_REQ_LINE_complete(connp);
        }
        if (*connp).in_current_read_offset < (*connp).in_current_len {
            (*connp).in_next_byte = *(*connp)
                .in_current_data
                .offset((*connp).in_current_read_offset as isize)
                as i32;
            (*connp).in_current_read_offset += 1;
            (*connp).in_stream_offset += 1
        } else {
            return Err(Status::DATA_BUFFER);
        }
        // Have we reached the end of the line?
        if (*connp).in_next_byte == '\n' as i32 {
            return htp_connp_REQ_LINE_complete(connp);
        }
    }
}

pub unsafe fn htp_connp_REQ_FINALIZE(connp: &mut htp_connection_parser::htp_connp_t) -> Result<()> {
    if (*connp).in_status != htp_connection_parser::htp_stream_state_t::HTP_STREAM_CLOSED {
        if (*connp).in_current_read_offset >= (*connp).in_current_len {
            (*connp).in_next_byte = -1
        } else {
            (*connp).in_next_byte = *(*connp)
                .in_current_data
                .offset((*connp).in_current_read_offset as isize)
                as i32
        }
        if (*connp).in_next_byte == -1 {
            return (*connp).state_request_complete().into();
        }
        if (*connp).in_next_byte != '\n' as i32
            || (*connp).in_current_consume_offset >= (*connp).in_current_read_offset
        {
            loop {
                //;i < max_read; i++) {
                if (*connp).in_current_read_offset < (*connp).in_current_len {
                    (*connp).in_next_byte = *(*connp)
                        .in_current_data
                        .offset((*connp).in_current_read_offset as isize)
                        as i32;
                    (*connp).in_current_read_offset += 1;
                    (*connp).in_stream_offset += 1
                } else {
                    return Err(Status::DATA_BUFFER);
                }
                // Have we reached the end of the line? For some reason
                // we can't test after IN_COPY_BYTE_OR_RETURN */
                if (*connp).in_next_byte == '\n' as i32 {
                    break;
                }
            }
        }
    }
    let mut data: *mut u8 = 0 as *mut u8;
    let mut len: usize = 0;
    htp_connp_req_consolidate_data(connp, &mut data, &mut len)?;
    if len == 0 {
        //closing
        return (*connp).state_request_complete().into();
    }
    let mut pos: usize = 0;
    let mut mstart: usize = 0;
    // skip past leading whitespace. IIS allows this
    while pos < len && htp_util::htp_is_space(*data.offset(pos as isize)) {
        pos = pos.wrapping_add(1)
    }
    if pos != 0 {
        mstart = pos
    }
    // The request method starts at the beginning of the
    // line and ends with the first whitespace character.
    while pos < len && !htp_util::htp_is_space(*data.offset(pos as isize)) {
        pos = pos.wrapping_add(1)
    }
    if pos <= mstart {
        //empty whitespace line
        let rc = (*connp)
            .in_tx_mut()
            .ok_or(Status::ERROR)?
            .req_process_body_data_ex(data as *const core::ffi::c_void, len);
        htp_connp_req_clear_buffer(connp);
        return rc;
    } else {
        let mut method_type = htp_method_t::HTP_M_UNKNOWN;
        let method: *mut bstr::bstr_t = bstr::bstr_dup_mem(
            data.offset(mstart as isize) as *const core::ffi::c_void,
            pos.wrapping_sub(mstart),
        );
        if !method.is_null() {
            method_type = htp_util::htp_convert_bstr_to_method(&*method);
            bstr::bstr_free(method);
        }
        if method_type == htp_method_t::HTP_M_UNKNOWN {
            // else continue
            // Interpret remaining bytes as body data
            htp_warn!(
                connp as *mut htp_connection_parser::htp_connp_t,
                htp_log_code::REQUEST_BODY_UNEXPECTED,
                "Unexpected request body"
            );
            let rc = (*connp)
                .in_tx_mut()
                .ok_or(Status::ERROR)?
                .req_process_body_data_ex(data as *const core::ffi::c_void, len);
            htp_connp_req_clear_buffer(connp);
            return rc;
        }
    }
    //unread last end of line so that REQ_LINE works
    if (*connp).in_current_read_offset < len as i64 {
        (*connp).in_current_read_offset = 0
    } else {
        (*connp).in_current_read_offset =
            ((*connp).in_current_read_offset as u64).wrapping_sub(len as u64) as i64
    }
    if (*connp).in_current_read_offset < (*connp).in_current_consume_offset {
        (*connp).in_current_consume_offset = (*connp).in_current_read_offset
    }
    (*connp).state_request_complete().into()
}

pub unsafe fn htp_connp_REQ_IGNORE_DATA_AFTER_HTTP_0_9(
    connp: &mut htp_connection_parser::htp_connp_t,
) -> Result<()> {
    // Consume whatever is left in the buffer.
    let bytes_left: usize = ((*connp).in_current_len - (*connp).in_current_read_offset) as usize;
    if bytes_left > 0 {
        (*connp).conn.flags |= htp_util::ConnectionFlags::HTP_CONN_HTTP_0_9_EXTRA
    }
    (*connp).in_current_read_offset =
        ((*connp).in_current_read_offset as u64).wrapping_add(bytes_left as u64) as i64;
    (*connp).in_current_consume_offset =
        ((*connp).in_current_consume_offset as u64).wrapping_add(bytes_left as u64) as i64;
    (*connp).in_stream_offset =
        ((*connp).in_stream_offset as u64).wrapping_add(bytes_left as u64) as i64;
    Err(Status::DATA)
}

/// The idle state is where the parser will end up after a transaction is processed.
/// If there is more data available, a new request will be started.
///
/// Returns HTP_OK on state change, HTP_ERROR on error, or HTP_DATA when more data is needed.
pub unsafe fn htp_connp_REQ_IDLE(connp: &mut htp_connection_parser::htp_connp_t) -> Result<()> {
    // We want to start parsing the next request (and change
    // the state from IDLE) only if there's at least one
    // byte of data available. Otherwise we could be creating
    // new structures even if there's no more data on the
    // connection.
    if (*connp).in_current_read_offset >= (*connp).in_current_len {
        return Err(Status::DATA);
    }

    if let Ok(tx_id) = (*connp).create_tx() {
        (*connp).set_in_tx_id(Some(tx_id))
    } else {
        return Err(Status::ERROR);
    }

    // Change state to TRANSACTION_START
    // Ignore the result.
    let _ = (*connp).state_request_start();
    Ok(())
}

/// Returns HTP_STREAM_DATA, HTP_STREAM_ERROR or STEAM_STATE_DATA_OTHER (see QUICK_START).
///         HTP_STREAM_CLOSED and HTP_STREAM_TUNNEL are also possible.
pub unsafe fn htp_connp_req_data(
    connp: &mut htp_connection_parser::htp_connp_t,
    timestamp: Option<htp_time_t>,
    data: *const core::ffi::c_void,
    len: usize,
) -> htp_connection_parser::htp_stream_state_t {
    // Return if the connection is in stop state.
    if (*connp).in_status == htp_connection_parser::htp_stream_state_t::HTP_STREAM_STOP {
        htp_info!(
            connp as *mut htp_connection_parser::htp_connp_t,
            htp_log_code::PARSER_STATE_ERROR,
            "Inbound parser is in HTP_STREAM_STOP"
        );
        return htp_connection_parser::htp_stream_state_t::HTP_STREAM_STOP;
    }
    // Return if the connection had a fatal error earlier
    if (*connp).in_status == htp_connection_parser::htp_stream_state_t::HTP_STREAM_ERROR {
        htp_error!(
            connp as *mut htp_connection_parser::htp_connp_t,
            htp_log_code::PARSER_STATE_ERROR,
            "Inbound parser is in HTP_STREAM_ERROR"
        );
        return htp_connection_parser::htp_stream_state_t::HTP_STREAM_ERROR;
    }
    // Sanity check: we must have a transaction pointer if the state is not IDLE (no inbound transaction)
    if (*connp).in_tx().is_none() && (*connp).in_state != State::IDLE {
        (*connp).in_status = htp_connection_parser::htp_stream_state_t::HTP_STREAM_ERROR;
        htp_error!(
            connp as *mut htp_connection_parser::htp_connp_t,
            htp_log_code::MISSING_INBOUND_TRANSACTION_DATA,
            "Missing inbound transaction data"
        );
        return htp_connection_parser::htp_stream_state_t::HTP_STREAM_ERROR;
    }
    // If the length of the supplied data chunk is zero, proceed
    // only if the stream has been closed. We do not allow zero-sized
    // chunks in the API, but we use them internally to force the parsers
    // to finalize parsing.
    if (data == 0 as *mut core::ffi::c_void || len == 0)
        && (*connp).in_status != htp_connection_parser::htp_stream_state_t::HTP_STREAM_CLOSED
    {
        htp_error!(
            connp as *mut htp_connection_parser::htp_connp_t,
            htp_log_code::ZERO_LENGTH_DATA_CHUNKS,
            "Zero-length data chunks are not allowed"
        );
        return htp_connection_parser::htp_stream_state_t::HTP_STREAM_CLOSED;
    }
    // Remember the timestamp of the current request data chunk
    if let Some(timestamp) = timestamp {
        (*connp).in_timestamp = timestamp;
    }

    // Store the current chunk information
    (*connp).in_current_data = data as *mut u8;
    (*connp).in_current_len = len as i64;
    (*connp).in_current_read_offset = 0;
    (*connp).in_current_consume_offset = 0;
    (*connp).in_current_receiver_offset = 0;
    (*connp).in_chunk_count = (*connp).in_chunk_count.wrapping_add(1);
    (*connp).conn.track_inbound_data(len);
    // Return without processing any data if the stream is in tunneling
    // mode (which it would be after an initial CONNECT transaction).
    if (*connp).in_status == htp_connection_parser::htp_stream_state_t::HTP_STREAM_TUNNEL {
        return htp_connection_parser::htp_stream_state_t::HTP_STREAM_TUNNEL;
    }
    if (*connp).out_status == htp_connection_parser::htp_stream_state_t::HTP_STREAM_DATA_OTHER {
        (*connp).out_status = htp_connection_parser::htp_stream_state_t::HTP_STREAM_DATA
    }
    loop
    // Invoke a processor, in a loop, until an error
    // occurs or until we run out of data. Many processors
    // will process a request, each pointing to the next
    // processor that needs to run.
    // Return if there's been an error or if we've run out of data. We are relying
    // on processors to supply error messages, so we'll keep quiet here.
    {
        let mut rc = (*connp).handle_in_state();
        if rc.is_ok() {
            if (*connp).in_status == htp_connection_parser::htp_stream_state_t::HTP_STREAM_TUNNEL {
                return htp_connection_parser::htp_stream_state_t::HTP_STREAM_TUNNEL;
            }
            rc = htp_req_handle_state_change(connp)
        }
        match rc {
            // Continue looping.
            Ok(_) => {}
            // Do we need more data?
            Err(Status::DATA) | Err(Status::DATA_BUFFER) => {
                // Ignore result.
                let _ = htp_connp_req_receiver_send_data(connp, false);
                if rc == Err(Status::DATA_BUFFER) && htp_connp_req_buffer(connp).is_err() {
                    (*connp).in_status =
                        htp_connection_parser::htp_stream_state_t::HTP_STREAM_ERROR;
                    return htp_connection_parser::htp_stream_state_t::HTP_STREAM_ERROR;
                }
                (*connp).in_status = htp_connection_parser::htp_stream_state_t::HTP_STREAM_DATA;
                return htp_connection_parser::htp_stream_state_t::HTP_STREAM_DATA;
            }
            // Check for suspended parsing.
            Err(Status::DATA_OTHER) => {
                // We might have actually consumed the entire data chunk?
                if (*connp).in_current_read_offset >= (*connp).in_current_len {
                    // Do not send STREAM_DATE_DATA_OTHER if we've consumed the entire chunk.
                    (*connp).in_status = htp_connection_parser::htp_stream_state_t::HTP_STREAM_DATA;
                    return htp_connection_parser::htp_stream_state_t::HTP_STREAM_DATA;
                } else {
                    // Partial chunk consumption.
                    (*connp).in_status =
                        htp_connection_parser::htp_stream_state_t::HTP_STREAM_DATA_OTHER;
                    return htp_connection_parser::htp_stream_state_t::HTP_STREAM_DATA_OTHER;
                }
            }
            // Check for the stop signal.
            Err(Status::STOP) => {
                (*connp).in_status = htp_connection_parser::htp_stream_state_t::HTP_STREAM_STOP;
                return htp_connection_parser::htp_stream_state_t::HTP_STREAM_STOP;
            }
            // Permanent stream error.
            Err(_) => {
                (*connp).in_status = htp_connection_parser::htp_stream_state_t::HTP_STREAM_ERROR;
                return htp_connection_parser::htp_stream_state_t::HTP_STREAM_ERROR;
            }
        }
    }
}
