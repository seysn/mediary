use bitflags::bitflags;
use error::H264Result;
use mediary_common::bitreader::BitReader;
use nal::{NalType, NalUnit};

pub mod error;
pub mod nal;

#[derive(Debug)]
pub struct AvcDecoderConfigurationRecord {
    pub profile: H264Profile,
    pub level: u8,
    pub nal_length_size: u8,
    pub sps: Vec<Sps>,
    pub pps: Vec<Pps>,
}

#[derive(Debug)]
pub struct Sps {
    pub profile_idc: u8,
    pub contraint_sets: u8,
    pub level: u8,
    pub parameter_id: u32,
    pub chroma_format: Option<u32>,
    pub separate_colour_plane: Option<bool>,
    pub bit_depth_luma_minus8: Option<u32>,
    pub bit_depth_chroma_minus8: Option<u32>,
    pub qpprime_y_zero_transform_bypass: Option<bool>,
    pub seq_scaling_matrix_present: Option<bool>,
    pub log2_max_frame_num_minus4: u32,
    pub pic_order_cnt_type: u32,
    pub log2_max_pic_order_cnt_lsb_minus4: Option<u32>,
    pub max_num_ref_frames: u32,
    pub gaps_in_frame_num_value_allowed: bool,
    pub pic_width_in_mbs_minus1: u32,
    pub pic_height_in_map_units_minus1: u32,
    pub direct_8x8_inference: bool,
    pub frame_crop_offsets: Option<FrameCropOffsets>,
    pub vui: Option<VuiParameters>,
}

#[derive(Debug)]
pub struct FrameCropOffsets {
    pub left: u32,
    pub right: u32,
    pub top: u32,
    pub bottom: u32,
}

#[derive(Debug)]
pub struct VuiParameters;

#[derive(Debug)]
pub struct Pps;

/// https://en.wikipedia.org/wiki/Advanced_Video_Coding#Profiles
#[derive(Debug)]
pub enum H264Profile {
    ConstrainedBaseline,
    Baseline,
    Extended,
    Main,
    High,
    ProgressiveHigh,
    ConstrainedHigh,
    High10,
    High422,
    High444,
    High10Intra,
    High422Intra,
    High444Intra,
    Cavlc444Intra,
    ScalableBaseline,
    ScalableConstrainedBaseline,
    ScalableHigh,
    ScalableContrainedHigh,
    ScalableHighIntra,
    StereoHigh,
    MultiviewHigh,
    MfcHigh,
    MfcDepthHigh,
    MultiviewDepthHigh,
    EnhancedMultiviewDepthHigh,
}

bitflags! {
    #[derive(Debug)]
    pub struct ConstraintSets: u8 {
        const Set0 = 0b1000_0000;
        const Set1 = 0b0100_0000;
        const Set2 = 0b0010_0000;
        const Set3 = 0b0001_0000;
        const Set4 = 0b0000_1000;
        const Set5 = 0b0000_0100;
    }
}

impl AvcDecoderConfigurationRecord {
    pub fn new(data: &[u8]) -> H264Result<Self> {
        let version = data[0];
        assert_eq!(version, 1);
        let profile = data[1];
        let constraint_sets = data[2];
        let level_indication = data[3];
        let length_size = (data[4] & 0b11) + 1;

        let num_of_sps = data[5] & 0x1f;
        let mut data = &data[6..];
        let mut sps = Vec::new();
        for _ in 0..num_of_sps {
            let sps_length =
                u16::from_be_bytes(data[..2].try_into().expect("slice has fewer than 2 bytes"))
                    as usize;

            data = &data[2..];
            let seq = NalUnit::from_raw(&data[..sps_length]);
            assert_eq!(seq.kind, NalType::Sps);
            sps.push(Sps::new(seq.payload)?);
            data = &data[sps_length..];
        }

        let num_of_pps = data[0] & 0x1f;
        data = &data[1..];
        let mut pps = Vec::new();
        for _ in 0..num_of_pps {
            let pps_length =
                u16::from_be_bytes(data[..2].try_into().expect("slice has fewer than 2 bytes"))
                    as usize;

            data = &data[2..];
            let seq = NalUnit::from_raw(&data[..pps_length]);
            assert_eq!(seq.kind, NalType::Pps);
            pps.push(Pps);
            data = &data[pps_length..];
        }

        Ok(Self {
            profile: H264Profile::new(profile, ConstraintSets::from_bits_truncate(constraint_sets)),
            level: level_indication,
            nal_length_size: length_size,
            sps,
            pps,
        })
    }
}

impl Sps {
    pub fn new(payload: &[u8]) -> H264Result<Self> {
        let mut reader = BitReader::new(payload);

        let profile_idc = reader.read_bits(8)? as u8;
        let contraint_sets = reader.read_bits(8)? as u8;
        let level = reader.read_bits(8)? as u8;
        let parameter_id = reader.read_ue()?;

        let mut chroma_format: Option<u32> = None;
        let mut separate_colour_plane: Option<bool> = None;
        let mut bit_depth_luma_minus8: Option<u32> = None;
        let mut bit_depth_chroma_minus8: Option<u32> = None;
        let mut qpprime_y_zero_transform_bypass: Option<bool> = None;
        let mut seq_scaling_matrix_present: Option<bool> = None;

        if matches!(
            profile_idc,
            100 | 110 | 122 | 244 | 44 | 83 | 86 | 118 | 128 | 138 | 139 | 134 | 135
        ) {
            chroma_format = Some(reader.read_ue()?);
            if let Some(3) = chroma_format {
                separate_colour_plane = Some(reader.read_flag()?);
            }

            bit_depth_luma_minus8 = Some(reader.read_ue()?);
            bit_depth_chroma_minus8 = Some(reader.read_ue()?);
            qpprime_y_zero_transform_bypass = Some(reader.read_flag()?);
            seq_scaling_matrix_present = Some(reader.read_flag()?);

            if let Some(true) = seq_scaling_matrix_present {
                todo!();
            }
        }

        let log2_max_frame_num_minus4 = reader.read_ue()?;
        let pic_order_cnt_type = reader.read_ue()?;
        let mut log2_max_pic_order_cnt_lsb_minus4: Option<u32> = None;
        if pic_order_cnt_type == 0 {
            log2_max_pic_order_cnt_lsb_minus4 = Some(reader.read_ue()?);
        } else if pic_order_cnt_type == 1 {
            todo!()
        }

        let max_num_ref_frames = reader.read_ue()?;
        let gaps_in_frame_num_value_allowed = reader.read_flag()?;
        let pic_width_in_mbs_minus1 = reader.read_ue()?;
        let pic_height_in_map_units_minus1 = reader.read_ue()?;
        let frame_mbs_only = reader.read_flag()?;
        if !frame_mbs_only {
            todo!()
        }

        let direct_8x8_inference = reader.read_flag()?;
        let frame_cropping = reader.read_flag()?;

        let frame_crop_offsets = if frame_cropping {
            let left = reader.read_ue()?;
            let right = reader.read_ue()?;
            let top = reader.read_ue()?;
            let bottom = reader.read_ue()?;

            Some(FrameCropOffsets {
                left,
                right,
                top,
                bottom,
            })
        } else {
            None
        };

        let vui_parameters_present: bool = reader.read_flag()?;
        let mut vui = None;
        if vui_parameters_present {
            vui = Some(VuiParameters);
        }

        Ok(Self {
            profile_idc,
            contraint_sets,
            level,
            parameter_id,
            chroma_format,
            separate_colour_plane,
            bit_depth_luma_minus8,
            bit_depth_chroma_minus8,
            qpprime_y_zero_transform_bypass,
            seq_scaling_matrix_present,
            log2_max_frame_num_minus4,
            pic_order_cnt_type,
            log2_max_pic_order_cnt_lsb_minus4,
            max_num_ref_frames,
            gaps_in_frame_num_value_allowed,
            pic_width_in_mbs_minus1,
            pic_height_in_map_units_minus1,
            direct_8x8_inference,
            frame_crop_offsets,
            vui,
        })
    }

    pub fn profile(&self) -> H264Profile {
        H264Profile::new(
            self.profile_idc,
            ConstraintSets::from_bits_truncate(self.contraint_sets),
        )
    }
}

impl H264Profile {
    fn new(profile: u8, sets: ConstraintSets) -> Self {
        match profile {
            66 => {
                if sets.contains(ConstraintSets::Set0) {
                    Self::ConstrainedBaseline
                } else {
                    Self::Baseline
                }
            }
            77 => Self::Main,
            88 => Self::Extended,
            100 => Self::High,
            110 => Self::High10,
            122 => Self::High422,
            244 => Self::High444,
            _ => todo!(),
        }
    }
}
