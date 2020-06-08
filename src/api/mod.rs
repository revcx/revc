use num_derive::FromPrimitive;
use num_traits::FromPrimitive;

pub const MAX_QP_TABLE_SIZE: usize = 58;
pub const MAX_QP_TABLE_SIZE_EXT: usize = 70;

/*****************************************************************************
 * return values and error code
 *****************************************************************************/
#[derive(FromPrimitive, ToPrimitive, PartialOrd, Ord, PartialEq, Eq)]
pub enum EvcStatus {
    /* no more frames, but it is OK */
    EVC_OK_NO_MORE_FRM = 205,
    /* progress success, but output is not available temporarily */
    EVC_OK_OUT_NOT_AVAILABLE = 204,
    /* frame dimension (width or height) has been changed */
    EVC_OK_DIM_CHANGED = (203),
    /* decoding success, but output frame has been delayed */
    EVC_OK_FRM_DELAYED = (202),
    /* not matched CRC value */
    EVC_ERR_BAD_CRC = (201),
    /* CRC value presented but ignored at decoder*/
    EVC_WARN_CRC_IGNORED = (200),

    EVC_OK = (0),

    EVC_ERR = (-1), /* generic error */
    EVC_ERR_INVALID_ARGUMENT = (-101),
    EVC_ERR_OUT_OF_MEMORY = (-102),
    EVC_ERR_REACHED_MAX = (-103),
    EVC_ERR_UNSUPPORTED = (-104),
    EVC_ERR_UNEXPECTED = (-105),
    EVC_ERR_UNSUPPORTED_COLORSPACE = (-201),
    EVC_ERR_MALFORMED_BITSTREAM = (-202),

    EVC_ERR_UNKNOWN = (-32767), /* unknown error */
}
/* return value checking *****************************************************/
#[inline]
pub fn evc_succeeded(ret: EvcStatus) -> bool {
    ret >= EvcStatus::EVC_OK
}

#[inline]
pub fn evc_failed(ret: EvcStatus) -> bool {
    ret < EvcStatus::EVC_OK
}

/*****************************************************************************
 * color spaces
 *****************************************************************************/
#[derive(FromPrimitive, ToPrimitive, PartialEq, Eq)]
pub enum EvcColorSpace {
    EVC_COLORSPACE_UNKNOWN = 0, /* unknown color space */

    /* YUV planar ****************************************************************/
    EVC_COLORSPACE_YUV_PLANAR_START = 100,

    /* YUV planar 8bit */
    EVC_COLORSPACE_YUV400 = 300, /* Y 8bit */
    EVC_COLORSPACE_YUV420 = 301, /* YUV420 8bit */
}

/*****************************************************************************
 * config types for decoder
 *****************************************************************************/
pub enum EvcdCfg {
    EVCD_CFG_SET_USE_PIC_SIGNATURE = 301,
}

/*****************************************************************************
 * config types for encoder
 *****************************************************************************/
pub enum EvceCfg {
    EVCE_CFG_SET_COMPLEXITY = 100,
    EVCE_CFG_SET_SPEED = 101,
    EVCE_CFG_SET_FORCE_OUT = 102,

    EVCE_CFG_SET_FINTRA = 200,
    EVCE_CFG_SET_QP = 201,
    EVCE_CFG_SET_BPS = 202,
    EVCE_CFG_SET_VBV_SIZE = 203,
    EVCE_CFG_SET_FPS = 204,
    EVCE_CFG_SET_I_PERIOD = 207,
    EVCE_CFG_SET_QP_MIN = 208,
    EVCE_CFG_SET_QP_MAX = 209,
    EVCE_CFG_SET_BU_SIZE = 210,
    EVCE_CFG_SET_USE_DEBLOCK = 211,
    EVCE_CFG_SET_DEBLOCK_A_OFFSET = 212,
    EVCE_CFG_SET_DEBLOCK_B_OFFSET = 213,
    EVCE_CFG_SET_USE_PIC_SIGNATURE = 301,
    EVCE_CFG_GET_COMPLEXITY = 500,
    EVCE_CFG_GET_SPEED = 501,
    EVCE_CFG_GET_QP_MIN = 600,
    EVCE_CFG_GET_QP_MAX = 601,
    EVCE_CFG_GET_QP = 602,
    EVCE_CFG_GET_RCT = 603,
    EVCE_CFG_GET_BPS = 604,
    EVCE_CFG_GET_FPS = 605,
    EVCE_CFG_GET_I_PERIOD = 608,
    EVCE_CFG_GET_BU_SIZE = 609,
    EVCE_CFG_GET_USE_DEBLOCK = 610,
    EVCE_CFG_GET_CLOSED_GOP = 611,
    EVCE_CFG_GET_HIERARCHICAL_GOP = 612,
    EVCE_CFG_GET_DEBLOCK_A_OFFSET = 613,
    EVCE_CFG_GET_DEBLOCK_B_OFFSET = 614,
    EVCE_CFG_GET_WIDTH = 701,
    EVCE_CFG_GET_HEIGHT = 702,
    EVCE_CFG_GET_RECON = 703,
}

/*****************************************************************************
 * NALU types
 *****************************************************************************/
pub enum EvcNut {
    EVC_NONIDR_NUT = 0,
    EVC_IDR_NUT = 1,
    EVC_SPS_NUT = 24,
    EVC_PPS_NUT = 25,
    EVC_APS_NUT = 26,
    EVC_SEI_NUT = 27,
}

/*****************************************************************************
 * slice type
 *****************************************************************************/
pub enum EvcSliceType {
    EVC_ST_UNKNOWN = 0,
    EVC_ST_I = 1,
    EVC_ST_P = 2,
    EVC_ST_B = 3,
}

/*****************************************************************************
 * type and macro for media time
 *****************************************************************************/
/* media time in 100-nanosec unit */
pub type EvcMtime = u64;

/*****************************************************************************
* image buffer format
*****************************************************************************
baddr
   +---------------------------------------------------+ ---
   |                                                   |  ^
   |                                              |    |  |
   |    a                                         v    |  |
   |   --- +-----------------------------------+ ---   |  |
   |    ^  |  (x, y)                           |  y    |  |
   |    |  |   +---------------------------+   + ---   |  |
   |    |  |   |                           |   |  ^    |  |
   |    |  |   |                           |   |  |    |  |
   |    |  |   |                           |   |  |    |  |
   |    |  |   |                           |   |  |    |  |
   |       |   |                           |   |       |
   |    ah |   |                           |   |  h    |  e
   |       |   |                           |   |       |
   |    |  |   |                           |   |  |    |  |
   |    |  |   |                           |   |  |    |  |
   |    |  |   |                           |   |  v    |  |
   |    |  |   +---------------------------+   | ---   |  |
   |    v  |                                   |       |  |
   |   --- +---+-------------------------------+       |  |
   |     ->| x |<----------- w ----------->|           |  |
   |       |<--------------- aw -------------->|       |  |
   |                                                   |  v
   +---------------------------------------------------+ ---

   |<---------------------- s ------------------------>|

*****************************************************************************/

pub const EVC_IMGB_MAX_PLANE: usize = 4;

pub struct EvcImgB {
    cs: EvcColorSpace, /* color space */
    np: usize,         /* number of plane */
    /* width (in unit of pixel) */
    w: [usize; EVC_IMGB_MAX_PLANE],
    /* height (in unit of pixel) */
    h: [usize; EVC_IMGB_MAX_PLANE],
    /* X position of left top (in unit of pixel) */
    x: [usize; EVC_IMGB_MAX_PLANE],
    /* Y postion of left top (in unit of pixel) */
    y: [usize; EVC_IMGB_MAX_PLANE],
    /* buffer stride (in unit of byte) */
    s: [usize; EVC_IMGB_MAX_PLANE],
    /* buffer elevation (in unit of byte) */
    e: [usize; EVC_IMGB_MAX_PLANE],
    /* address of each plane */
    //void              * a[EVC_IMGB_MAX_PLANE];

    /* time-stamps */
    ts: [EvcMtime; 4],

    ndata: [isize; 4], /* arbitrary data, if needs */
    //void              * pdata[4]; /* arbitrary adedress if needs */

    /* aligned width (in unit of pixel) */
    aw: [usize; EVC_IMGB_MAX_PLANE],
    /* aligned height (in unit of pixel) */
    ah: [usize; EVC_IMGB_MAX_PLANE],

    /* left padding size (in unit of pixel) */
    padl: [usize; EVC_IMGB_MAX_PLANE],
    /* right padding size (in unit of pixel) */
    padr: [usize; EVC_IMGB_MAX_PLANE],
    /* up padding size (in unit of pixel) */
    padu: [usize; EVC_IMGB_MAX_PLANE],
    /* bottom padding size (in unit of pixel) */
    padb: [usize; EVC_IMGB_MAX_PLANE],

    /* address of actual allocated buffer */
    //void              * baddr[EVC_IMGB_MAX_PLANE];
    /* actual allocated buffer size */
    bsize: [usize; EVC_IMGB_MAX_PLANE],

    /* life cycle management */
    /*int                 refcnt;
    int                 (*addref)(EVC_IMGB * imgb);
    int                 (*getref)(EVC_IMGB * imgb);
    int                 (*release)(EVC_IMGB * imgb);*/
    crop_idx: usize,
    crop_l: usize,
    crop_r: usize,
    crop_t: usize,
    crop_b: usize,
}

/*****************************************************************************
 * Bitstream buffer
 *****************************************************************************/
pub struct EvcBitB {
    /* user space address indicating buffer */
    //void              * addr;
    /* physical address indicating buffer, if any */
    //void              * pddr;
    /* byte size of buffer memory */
    bsize: usize,
    /* byte size of bitstream in buffer */
    ssize: usize,
    /* bitstream has an error? */
    err: EvcStatus,
    /* arbitrary data, if needs */
    ndata: [usize; 4],
    /* arbitrary address, if needs */
    //void              * pdata[4];
    /* time-stamps */
    ts: [EvcMtime; 4],
}

/*****************************************************************************
 * description for creating of decoder
 *****************************************************************************/
pub struct EvcdCdsc {
    na: isize, /* nothing */
}

/*****************************************************************************
 * status after decoder operation
 *****************************************************************************/
pub struct EvcdStat {
    /* byte size of decoded bitstream (read size of bitstream) */
    read: isize,
    /* nalu type */
    nalu_type: isize,
    /* slice type */
    stype: isize,
    /* frame number monotonically increased whenever decoding a frame.
    note that it has negative value if the decoded data is not frame */
    fnum: isize,
    /* picture order count */
    poc: isize,
    /* layer id */
    tid: isize,

    /* number of reference pictures */
    refpic_num: [u8; 2],
    /* list of reference pictures */
    refpic: [[usize; 2]; 16],
}

pub const MAX_NUM_REF_PICS: usize = 21;
pub const MAX_NUM_ACTIVE_REF_FRAME: usize = 5;
pub const MAX_NUM_RPLS: usize = 32;

/* rpl structure */
pub struct EvcRpl {
    poc: isize,
    tid: isize,
    ref_pic_num: isize,
    ref_pic_active_num: isize,
    ref_pics: [isize; MAX_NUM_REF_PICS],
    pic_type: u8,
}

/* chromaQP table structure to be signalled in SPS*/
pub struct EvcChromaTable {
    chroma_qp_table_present_flag: bool,
    same_qp_table_for_chroma: bool,
    global_offset_flag: bool,
    num_points_in_qp_table_minus1: [isize; 2],
    delta_qp_in_val_minus1: [[isize; 2]; MAX_QP_TABLE_SIZE],
    delta_qp_out_val: [[isize; 2]; MAX_QP_TABLE_SIZE],
}

/*****************************************************************************
 * description for creating of encoder
 *****************************************************************************/
pub struct EvceCDSC {
    //TODO:
}

/*****************************************************************************
 * status after encoder operation
 *****************************************************************************/
pub struct EvceStat {
    /* encoded bitstream byte size */
    write: usize,
    /* encoded sei messages byte size */
    sei_size: usize,
    /* picture number increased whenever encoding a frame */
    fnum: usize,
    /* nalu type */
    nalu_type: EvcNut,
    /* slice type */
    stype: EvcSliceType,
    /* quantization parameter used for encoding */
    qp: isize,
    /* picture order count */
    poc: isize,
    /* layer id */
    tid: isize,
    /* number of reference pictures */
    refpic_num: [isize; 2],
    /* list of reference pictures */
    refpic: [[isize; 2]; 16],
}
