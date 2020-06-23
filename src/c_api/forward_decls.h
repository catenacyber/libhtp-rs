/*
 * Forward declarations of needed structs
 */
typedef struct htp_cfg_t htp_cfg_t;
typedef struct htp_conn_t htp_conn_t;
typedef struct htp_connp_t htp_connp_t;
typedef struct htp_file_t htp_file_t;
typedef struct htp_file_data_t htp_file_data_t;
typedef struct htp_header_t htp_header_t;
typedef struct htp_header_line_t htp_header_line_t;
typedef struct htp_param_t htp_param_t;
typedef struct htp_mpartp_t htp_mpartp_t;
typedef struct htp_tx_data_t htp_tx_data_t;
typedef struct htp_tx_t htp_tx_t;
typedef struct htp_uri_t htp_uri_t;
typedef struct bstr_t bstr;
typedef struct htp_table_t htp_table_t;
/* Needed to suppress type issues in the exposed
 * decompression state. Will go away when decompression
 * is cleaned up.
 */
typedef struct internal_state internal_state;

/* Declare timeval as struct timeval, so it can be
 * re-exported as htp_time_t. Also will be cleaned up
 * when we sort the timeval business.
 */
typedef struct timeval timeval;

/* These are needed to map the C flag names into the cbindgen
 * struct name prefixed versions
 */
typedef uint64_t Flags;
#define HTP_FIELD_UNPARSEABLE       0x000000004ULL
#define HTP_FIELD_INVALID           0x000000008ULL
#define HTP_FIELD_FOLDED            0x000000010ULL
#define HTP_FIELD_REPEATED          0x000000020ULL
#define HTP_FIELD_LONG              0x000000040ULL
#define HTP_FIELD_RAW_NUL           0x000000080ULL
#define HTP_REQUEST_SMUGGLING       0x000000100ULL
#define HTP_INVALID_FOLDING         0x000000200ULL
#define HTP_REQUEST_INVALID_T_E     0x000000400ULL
#define HTP_MULTI_PACKET_HEAD       0x000000800ULL
#define HTP_HOST_MISSING            0x000001000ULL
#define HTP_HOST_AMBIGUOUS          0x000002000ULL
#define HTP_PATH_ENCODED_NUL        0x000004000ULL
#define HTP_PATH_RAW_NUL            0x000008000ULL
#define HTP_PATH_INVALID_ENCODING   0x000010000ULL
#define HTP_PATH_INVALID            0x000020000ULL
#define HTP_PATH_OVERLONG_U         0x000040000ULL
#define HTP_PATH_ENCODED_SEPARATOR  0x000080000ULL
#define HTP_PATH_UTF8_VALID         0x000100000ULL
#define HTP_PATH_UTF8_INVALID       0x000200000ULL
#define HTP_PATH_UTF8_OVERLONG      0x000400000ULL
#define HTP_PATH_HALF_FULL_RANGE    0x000800000ULL
#define HTP_STATUS_LINE_INVALID     0x001000000ULL
#define HTP_HOSTU_INVALID           0x002000000ULL
#define HTP_HOSTH_INVALID           0x004000000ULL
#define HTP_URLEN_ENCODED_NUL       0x008000000ULL
#define HTP_URLEN_INVALID_ENCODING  0x010000000ULL
#define HTP_URLEN_OVERLONG_U        0x020000000ULL
#define HTP_URLEN_HALF_FULL_RANGE   0x040000000ULL
#define HTP_URLEN_RAW_NUL           0x080000000ULL
#define HTP_REQUEST_INVALID         0x100000000ULL
#define HTP_REQUEST_INVALID_C_L     0x200000000ULL
#define HTP_AUTH_INVALID            0x400000000ULL
#define HTP_HOST_INVALID            (HTP_HOSTU_INVALID | HTP_HOSTH_INVALID)

#define HTP_PROTOCOL_INVALID PROTOCOL_INVALID
#define HTP_PROTOCOL_UNKNOWN PROTOCOL_UNKNOWN
#define HTP_PROTOCOL_0_9  PROTOCOL_V0_9
#define HTP_PROTOCOL_1_0  PROTOCOL_V1_0
#define HTP_PROTOCOL_1_1  PROTOCOL_V1_1

#define HTP_ERROR_RESERVED STATUS_ERROR_RESERVED
#define HTP_ERROR STATUS_ERROR
#define HTP_DECLINED STATUS_DECLINED
#define HTP_OK STATUS_OK
#define HTP_DATA STATUS_DATA
#define HTP_DATA_OTHER STATUS_DATA_OTHER
#define HTP_STOP STATUS_STOP
#define HTP_DATA_BUFFER STATUS_DATA_BUFFER
#define HTP_STATUS_RESERVED STATUS_STATUS_RESERVED
