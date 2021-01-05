use crate::{
    bstr::Bstr,
    config::HtpUnwanted,
    connection_parser::ConnectionParser,
    error::Result,
    headers::{headers, Flags as HeaderFlags},
    parsers::{parse_content_length, parse_protocol},
    request::HtpMethod,
    transaction::{Header, HtpProtocol},
    util::{
        convert_to_method, is_space, take_ascii_whitespace, take_is_space, take_not_is_space,
        take_until_null, FlagOperations, HtpFlags,
    },
};
use nom::{bytes::complete::take_while, error::ErrorKind, sequence::tuple};
use std::cmp::Ordering;

impl ConnectionParser {
    /// Extract one request header. A header can span multiple lines, in
    /// which case they will be folded into one before parsing is attempted.
    ///
    /// Returns OK or ERROR
    fn process_request_header_generic(&mut self, header: Header) -> Result<()> {
        // Try to parse the header.
        let mut repeated = false;
        let reps = self.in_tx_mut_ok()?.req_header_repetitions;
        let mut update_reps = false;
        // Do we already have a header with the same name?
        if let Some((_, h_existing)) = self
            .in_tx_mut_ok()?
            .request_headers
            .get_nocase_mut(header.name.as_slice())
        {
            // TODO Do we want to have a list of the headers that are
            //      allowed to be combined in this way?
            if !h_existing.flags.is_set(HtpFlags::FIELD_REPEATED) {
                // This is the second occurence for this header.
                repeated = true;
            } else if reps < 64 {
                update_reps = true;
            } else {
                return Ok(());
            }
            // For simplicity reasons, we count the repetitions of all headers
            // Keep track of repeated same-name headers.
            h_existing.flags.set(HtpFlags::FIELD_REPEATED);
            // Having multiple C-L headers is against the RFC but
            // servers may ignore the subsequent headers if the values are the same.
            if header.name.cmp_nocase("Content-Length") == Ordering::Equal {
                // Don't use string comparison here because we want to
                // ignore small formatting differences.
                let existing_cl = parse_content_length(&h_existing.value, None);
                let new_cl = parse_content_length(&header.value, None);
                // Ambiguous response C-L value.
                if existing_cl.is_none() || new_cl.is_none() || existing_cl != new_cl {
                    htp_warn!(
                        self,
                        HtpLogCode::DUPLICATE_CONTENT_LENGTH_FIELD_IN_REQUEST,
                        "Ambiguous request C-L value"
                    );
                }
            } else {
                // Add to the existing header.
                h_existing.value.extend_from_slice(b", ");
                h_existing.value.extend_from_slice(header.value.as_slice());
            }
        } else {
            self.in_tx_mut_ok()?
                .request_headers
                .add(header.name.clone(), header);
        }
        if update_reps {
            self.in_tx_mut_ok()?.req_header_repetitions =
                self.in_tx_mut_ok()?.req_header_repetitions.wrapping_add(1)
        }
        if repeated {
            htp_warn!(
                self,
                HtpLogCode::REQUEST_HEADER_REPETITION,
                "Repetition for header"
            );
        }
        Ok(())
    }

    /// Generic request header parser.
    pub fn process_request_headers_generic<'a>(
        &mut self,
        data: &'a [u8],
    ) -> Result<(&'a [u8], bool)> {
        let rc = headers(data);
        if let Ok((remaining, (headers, eoh))) = rc {
            for h in headers {
                let mut flags = 0;
                let name_flags = h.name.flags;
                // Ignore LWS after field-name.
                if name_flags.is_set(HeaderFlags::NAME_TRAILING_WHITESPACE) {
                    // Log only once per transaction.
                    htp_warn_once!(
                        self,
                        HtpLogCode::REQUEST_INVALID_LWS_AFTER_NAME,
                        "Request field invalid: LWS after name",
                        self.in_tx_mut_ok()?.flags,
                        flags,
                        HtpFlags::FIELD_INVALID
                    );
                }
                //If name has leading whitespace, probably invalid folding
                if name_flags.is_set(HeaderFlags::NAME_LEADING_WHITESPACE) {
                    // Invalid folding.
                    // Warn only once per transaction.
                    htp_warn_once!(
                        self,
                        HtpLogCode::INVALID_REQUEST_FIELD_FOLDING,
                        "Invalid request field folding",
                        self.in_tx_mut_ok()?.flags,
                        flags,
                        HtpFlags::INVALID_FOLDING
                    );
                }
                // Check that field-name is a token
                if name_flags.is_set(HeaderFlags::NAME_NON_TOKEN_CHARS) {
                    // Incorrectly formed header name.
                    // Log only once per transaction.
                    htp_warn_once!(
                        self,
                        HtpLogCode::REQUEST_HEADER_INVALID,
                        "Request header name is not a token",
                        self.in_tx_mut_ok()?.flags,
                        flags,
                        HtpFlags::FIELD_INVALID
                    );
                }
                // No colon?
                if name_flags.is_set(HeaderFlags::MISSING_COLON) {
                    // Log only once per transaction.
                    // We handle this case as a header with an empty name, with the value equal
                    // to the entire input string.
                    // TODO Apache will respond to this problem with a 400.
                    // Now extract the name and the value
                    htp_warn_once!(
                        self,
                        HtpLogCode::REQUEST_FIELD_MISSING_COLON,
                        "Request field invalid: colon missing",
                        self.in_tx_mut_ok()?.flags,
                        flags,
                        HtpFlags::FIELD_UNPARSEABLE
                    );
                } else if name_flags.is_set(HeaderFlags::NAME_EMPTY) {
                    // Empty header name.
                    // Log only once per transaction.
                    htp_warn_once!(
                        self,
                        HtpLogCode::REQUEST_INVALID_EMPTY_NAME,
                        "Request field invalid: empty name",
                        self.in_tx_mut_ok()?.flags,
                        flags,
                        HtpFlags::FIELD_INVALID
                    );
                }
                self.process_request_header_generic(Header::new_with_flags(
                    h.name.name.into(),
                    h.value.value.into(),
                    flags,
                ))?;
            }
            Ok((remaining, eoh))
        } else {
            Ok((data, false))
        }
    }

    pub fn parse_request_line_generic_ex(
        &mut self,
        request_line: &[u8],
        nul_terminates: bool,
    ) -> Result<()> {
        let mut mstart: bool = false;
        let mut data: &[u8] = request_line;
        if nul_terminates {
            if let Ok((_, before_null)) = take_until_null(data) {
                data = before_null
            }
        }

        // The request method starts at the beginning of the
        // line and ends with the first whitespace character.
        let method_parser = tuple::<_, _, (_, ErrorKind), _>
                                // skip past leading whitespace. IIS allows this
                               ((take_is_space,
                               take_not_is_space,
                                // Ignore whitespace after request method. The RFC allows
                                 // for only one SP, but then suggests any number of SP and HT
                                 // should be permitted. Apache uses isspace(), which is even
                                 // more permitting, so that's what we use here.
                               take_ascii_whitespace()
                               ));

        if let Ok((remaining, (ls, method, ws))) = method_parser(data) {
            if !ls.is_empty() {
                htp_warn!(
                    self,
                    HtpLogCode::REQUEST_LINE_LEADING_WHITESPACE,
                    "Request line: leading whitespace"
                );

                let requestline_leading_whitespace_unwanted =
                    self.cfg.requestline_leading_whitespace_unwanted;
                if requestline_leading_whitespace_unwanted != HtpUnwanted::IGNORE {
                    // reset mstart so that we copy the whitespace into the method
                    mstart = true;
                    // set expected response code to this anomaly
                    self.in_tx_mut_ok()?.response_status_expected_number =
                        requestline_leading_whitespace_unwanted
                }
            }

            if mstart {
                self.in_tx_mut_ok()?.request_method =
                    Some(Bstr::from([&ls[..], &method[..]].concat()));
            } else {
                self.in_tx_mut_ok()?.request_method = Some(Bstr::from(method));
            }

            if let Some(request_method) = &self.in_tx_mut_ok()?.request_method {
                self.in_tx_mut_ok()?.request_method_number =
                    convert_to_method(request_method.as_slice());
            }

            // Too much performance overhead for fuzzing
            if ws.iter().any(|&c| c != 0x20) {
                htp_warn!(
                    self,
                    HtpLogCode::METHOD_DELIM_NON_COMPLIANT,
                    "Request line: non-compliant delimiter between Method and URI"
                );
            }

            if remaining.is_empty() {
                // No, this looks like a HTTP/0.9 request.
                self.in_tx_mut_ok()?.is_protocol_0_9 = true;
                self.in_tx_mut_ok()?.request_protocol_number = HtpProtocol::V0_9;
                if self.in_tx_mut_ok()?.request_method_number == HtpMethod::UNKNOWN {
                    htp_warn!(
                        self,
                        HtpLogCode::REQUEST_LINE_UNKNOWN_METHOD,
                        "Request line: unknown method only"
                    );
                }
                return Ok(());
            }

            let uri_protocol_parser = tuple::<_, _, (_, ErrorKind), _>
            // The URI ends with the first whitespace.
            ((take_while(|c: u8| c != 0x20),
              // Ignore whitespace after URI.
              take_is_space)
            );

            if let Ok((mut protocol, (mut uri, _))) = uri_protocol_parser(remaining) {
                if uri.len() == remaining.len() && uri.iter().any(|&c| is_space(c)) {
                    // warn regardless if we've seen non-compliant chars
                    htp_warn!(
                        self,
                        HtpLogCode::URI_DELIM_NON_COMPLIANT,
                        "Request line: URI contains non-compliant delimiter"
                    );
                    // if we've seen some 'bad' delimiters, we retry with those
                    let uri_protocol_parser2 =
                        tuple::<_, _, (_, ErrorKind), _>((take_not_is_space, take_is_space));
                    if let Ok((protocol2, (uri2, _))) = uri_protocol_parser2(remaining) {
                        uri = uri2;
                        protocol = protocol2;
                    }
                }
                self.in_tx_mut_ok()?.request_uri = Some(Bstr::from(uri));
                // Is there protocol information available?
                if protocol.is_empty() {
                    // No, this looks like a HTTP/0.9 request.
                    self.in_tx_mut_ok()?.is_protocol_0_9 = true;
                    self.in_tx_mut_ok()?.request_protocol_number = HtpProtocol::V0_9;
                    if self.in_tx_mut_ok()?.request_method_number == HtpMethod::UNKNOWN {
                        htp_warn!(
                            self,
                            HtpLogCode::REQUEST_LINE_UNKNOWN_METHOD_NO_PROTOCOL,
                            "Request line: unknown method and no protocol"
                        );
                    }
                    return Ok(());
                }
                // The protocol information continues until the end of the line.
                self.in_tx_mut_ok()?.request_protocol = Some(Bstr::from(protocol));
                self.in_tx_mut_ok()?.request_protocol_number = parse_protocol(protocol, self);
                if self.in_tx_mut_ok()?.request_method_number == HtpMethod::UNKNOWN
                    && self.in_tx_mut_ok()?.request_protocol_number == HtpProtocol::INVALID
                {
                    htp_warn!(
                        self,
                        HtpLogCode::REQUEST_LINE_UNKNOWN_METHOD_INVALID_PROTOCOL,
                        "Request line: unknown method and invalid protocol"
                    );
                }
            }
        }
        Ok(())
    }
}
