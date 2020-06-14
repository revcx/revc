use super::EvcdBsr;
use crate::api::EvcError;
use crate::com::*;

use log::*;

pub(crate) fn evcd_eco_nalu(bs: &mut EvcdBsr) -> Result<EvcNalu, EvcError> {
    let mut nalu = EvcNalu::default();

    //nalu->nal_unit_size = evc_bsr_read(bs, 32);
    nalu.forbidden_zero_bit = bs.read(1) as u8;

    if nalu.forbidden_zero_bit != 0 {
        error!("malformed bitstream: forbidden_zero_bit != 0\n");
        return Err(EvcError::EVC_ERR_MALFORMED_BITSTREAM);
    }

    nalu.nal_unit_type = bs.read(6) as u8 - 1;
    nalu.nuh_temporal_id = bs.read(3) as u8;
    nalu.nuh_reserved_zero_5bits = bs.read(5) as u8;

    if nalu.nuh_reserved_zero_5bits != 0 {
        error!("malformed bitstream: nuh_reserved_zero_5bits != 0");
        return Err(EvcError::EVC_ERR_MALFORMED_BITSTREAM);
    }

    nalu.nuh_extension_flag = bs.read(1) != 0;

    if nalu.nuh_extension_flag {
        error!("malformed bitstream: nuh_extension_flag != 0");
        return Err(EvcError::EVC_ERR_MALFORMED_BITSTREAM);
    }

    Ok(nalu)
}
