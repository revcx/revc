use super::*;

#[rustfmt::skip]
pub(crate) static tbl_slice_depth_P: [[u8;16];5] =
[
    /* gop_size = 2 */
    [ FRM_DEPTH_2, FRM_DEPTH_1, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF ],
    /* gop_size = 4 */
    [ FRM_DEPTH_3, FRM_DEPTH_2, FRM_DEPTH_3, FRM_DEPTH_1, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF ],
    /* gop_size = 8 */
    [ FRM_DEPTH_4, FRM_DEPTH_3, FRM_DEPTH_4, FRM_DEPTH_2, FRM_DEPTH_4, FRM_DEPTH_3, FRM_DEPTH_4, FRM_DEPTH_1,
        0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF ],
    /* gop_size = 12 */
    [0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF],
    /* gop_size = 16 */
    [ FRM_DEPTH_5, FRM_DEPTH_4, FRM_DEPTH_5, FRM_DEPTH_3, FRM_DEPTH_5, FRM_DEPTH_4, FRM_DEPTH_5, FRM_DEPTH_2,
        FRM_DEPTH_5, FRM_DEPTH_4, FRM_DEPTH_5, FRM_DEPTH_3, FRM_DEPTH_5, FRM_DEPTH_4, FRM_DEPTH_5, FRM_DEPTH_1 ],
];

#[rustfmt::skip]
pub(crate) static tbl_slice_depth: [[u8;15];5] =
[
    /* gop_size = 2 */
    [ FRM_DEPTH_2, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF ],
    /* gop_size = 4 */
    [ FRM_DEPTH_2, FRM_DEPTH_3, FRM_DEPTH_3, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF ],
    /* gop_size = 8 */
    [ FRM_DEPTH_2, FRM_DEPTH_3, FRM_DEPTH_3, FRM_DEPTH_4, FRM_DEPTH_4, FRM_DEPTH_4, FRM_DEPTH_4,
        0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF ],
    /* gop_size = 12 */
    [0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF ],
    /* gop_size = 16 */
    [ FRM_DEPTH_2, FRM_DEPTH_3, FRM_DEPTH_3, FRM_DEPTH_4, FRM_DEPTH_4, FRM_DEPTH_4, FRM_DEPTH_4, FRM_DEPTH_5,
        FRM_DEPTH_5,  FRM_DEPTH_5, FRM_DEPTH_5, FRM_DEPTH_5, FRM_DEPTH_5, FRM_DEPTH_5, FRM_DEPTH_5 ],
];

pub(crate) struct QP_ADAPT_PARAM {
    pub(crate) qp_offset_layer: i8,
    pub(crate) qp_offset_model_offset: f64,
    pub(crate) qp_offset_model_scale: f64,
}

#[rustfmt::skip]
pub(crate) static  qp_adapt_param_ra:[QP_ADAPT_PARAM;8] =
[
    QP_ADAPT_PARAM{ qp_offset_layer: -3, qp_offset_model_offset:  0.0000, qp_offset_model_scale: 0.0000},
    QP_ADAPT_PARAM{ qp_offset_layer:  1, qp_offset_model_offset:  0.0000, qp_offset_model_scale: 0.0000},
    QP_ADAPT_PARAM{ qp_offset_layer:  1, qp_offset_model_offset: -4.8848, qp_offset_model_scale: 0.2061},
    QP_ADAPT_PARAM{ qp_offset_layer:  4, qp_offset_model_offset: -5.7476, qp_offset_model_scale: 0.2286},
    QP_ADAPT_PARAM{ qp_offset_layer:  5, qp_offset_model_offset: -5.9000, qp_offset_model_scale: 0.2333},
    QP_ADAPT_PARAM{ qp_offset_layer:  6, qp_offset_model_offset: -7.1444, qp_offset_model_scale: 0.3000},
    QP_ADAPT_PARAM{ qp_offset_layer:  7, qp_offset_model_offset: -7.1444, qp_offset_model_scale: 0.3000},
    QP_ADAPT_PARAM{ qp_offset_layer:  8, qp_offset_model_offset: -7.1444, qp_offset_model_scale: 0.3000},
];

#[rustfmt::skip]
pub(crate) static   qp_adapt_param_ld:[QP_ADAPT_PARAM;8] =
[
    QP_ADAPT_PARAM{ qp_offset_layer: -1, qp_offset_model_offset:  0.0000, qp_offset_model_scale:  0.0000 },
    QP_ADAPT_PARAM{ qp_offset_layer:  1, qp_offset_model_offset:  0.0000, qp_offset_model_scale:  0.0000 },
    QP_ADAPT_PARAM{ qp_offset_layer:  4, qp_offset_model_offset: -6.5000, qp_offset_model_scale:  0.2590 },
    QP_ADAPT_PARAM{ qp_offset_layer:  4, qp_offset_model_offset: -6.5000, qp_offset_model_scale:  0.2590 },
    QP_ADAPT_PARAM{ qp_offset_layer:  5, qp_offset_model_offset: -6.5000, qp_offset_model_scale:  0.2590 },
    QP_ADAPT_PARAM{ qp_offset_layer:  5, qp_offset_model_offset: -6.5000, qp_offset_model_scale:  0.2590 },
    QP_ADAPT_PARAM{ qp_offset_layer:  5, qp_offset_model_offset: -6.5000, qp_offset_model_scale:  0.2590 },
    QP_ADAPT_PARAM{ qp_offset_layer:  5, qp_offset_model_offset: -6.5000, qp_offset_model_scale:  0.2590 },
];

#[rustfmt::skip]
pub(crate) static   qp_adapt_param_ai:[QP_ADAPT_PARAM;8] =
[
    QP_ADAPT_PARAM{ qp_offset_layer: 0, qp_offset_model_offset: 0.0, qp_offset_model_scale: 0.0},
    QP_ADAPT_PARAM{ qp_offset_layer: 0, qp_offset_model_offset: 0.0, qp_offset_model_scale: 0.0},
    QP_ADAPT_PARAM{ qp_offset_layer: 0, qp_offset_model_offset: 0.0, qp_offset_model_scale: 0.0},
    QP_ADAPT_PARAM{ qp_offset_layer: 0, qp_offset_model_offset: 0.0, qp_offset_model_scale: 0.0},
    QP_ADAPT_PARAM{ qp_offset_layer: 0, qp_offset_model_offset: 0.0, qp_offset_model_scale: 0.0},
    QP_ADAPT_PARAM{ qp_offset_layer: 0, qp_offset_model_offset: 0.0, qp_offset_model_scale: 0.0},
    QP_ADAPT_PARAM{ qp_offset_layer: 0, qp_offset_model_offset: 0.0, qp_offset_model_scale: 0.0},
    QP_ADAPT_PARAM{ qp_offset_layer: 0, qp_offset_model_offset: 0.0, qp_offset_model_scale: 0.0},
];
