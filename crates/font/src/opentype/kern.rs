use crate::{parsers::*, opentype::gpos::KernTable, FontError};
use nom::{
    bytes::complete::take, combinator::map, number::complete::{be_i16, be_u16, be_u32, be_u8}, sequence::tuple, Parser
};

fn parse_kern_format0<'a>(i: &'a [u8], table: &mut KernTable) -> Result<(), FontError> {
    let (i, n_pairs) = be_u16(i)?;
    let (i, _search_range) = be_u16(i)?;
    let (i, _entry_selector) = be_u16(i)?;
    let (i, _range_shift) = be_u16(i)?;
    fn _cu_parser<'a>(s: &'a [u8]) -> nom::IResult<&'a [u8], (u16, u16, i16)> {
        let (s, sx) = be_u16(s)?;
        let (s, sy) = be_u16(s)?;
        let (s, sz) = be_i16(s)?;
        Ok((s, (sx, sy, sz)))
    }

    table.glyph_pairs.reserve(n_pairs as usize);
    for (left, right, kern) in iterator(i, _cu_parser).take(n_pairs as usize) {
        table.glyph_pairs.insert((left, right), kern);
    }
    Ok(())
}

fn parse_kern_format2<'a>(data: &'a [u8], table: &mut KernTable) -> Result<(), FontError> {
    let (i, _row_width) = be_u16(data)?;
    let (i, left_class_table_off) = be_u16(i)?;
    let (i, right_class_table_off) = be_u16(i)?;
    let (i, _array_off) = be_u16(i)?;
    
    let num_left = be_u16(slice!(data, left_class_table_off as usize + 2 ..))?.1 as usize;
    let num_right = be_u16(slice!(data, right_class_table_off as usize + 2 ..))?.1 as usize;
    table.glyph_pairs.reserve(num_left * num_right);

    let class_table = |off| {
        let (i, first_glyph) = be_u16(slice!(data, off as usize ..))?;
        let (i, n_glyphs) = be_u16(i)?;
        let (_, offsets) = take(n_glyphs as usize * 2)(i)?;
        Ok((first_glyph ..).zip(iterator(offsets, |s| map(be_u16, |n| n as usize).parse(s))))
    };
    for (left_gid, left_off) in class_table(left_class_table_off)? {
        for (right_gid, right_off) in class_table(right_class_table_off)? {
            let off = left_off + right_off;
            let (_, kern) = be_i16(slice!(data, off .. off + 2))?;
            table.glyph_pairs.insert((left_gid, right_gid), kern);
        }
    }
    Ok(())
}

pub fn parse_kern(input: &[u8]) -> Result<KernTable, FontError> {
    let (_i, version) = be_u16(input)?;
    debug!("kern table version {}", version);
    match version {
        0 => parse_kern_ms(input),
        1 => parse_kern_apple(input),
        _ => error!("stahp!")
    }
}

pub fn parse_kern_apple(i: &[u8]) -> Result<KernTable, FontError> {
    let (i, version) = be_u32(i)?;
    require_eq!(version, 0x00010000);
    
    let mut table = KernTable::default();
    let (mut input, n_tables) = be_u32(i)?;
    for _ in 0 .. n_tables {
        let (i, length) = be_u32(i)?;
        let (i, coverage) = be_u8(i)?;
        let (i, format) = be_u8(i)?;
        let (i, tuple_index) = be_u16(i)?;

        let (new_input, data) = take(length)(i)?;
        match format {
            0 => apple_kern_0(i, &mut table)?,
            _ => error!("unimplemented apple kern format {}", format),
        }
        input = new_input;
    }
    Ok(table)
}

fn apple_kern_0(i: &[u8], table: &mut KernTable) -> Result<(), FontError> {
    let (i, n_pairs) = be_u16(i)?;
    let (i, _) = be_u16(i)?;
    let (i, _) = be_u16(i)?;
    let (i, _) = be_u16(i)?;

    for _ in 0 .. n_pairs {
        let (i, left) = be_u16(i)?;
        let (i, right) = be_u16(i)?;
        let (i, value) = be_i16(i)?;
        table.glyph_pairs.insert((left, right), value);
    }

    Ok(())
}

pub fn parse_kern_ms(i: &[u8]) -> Result<KernTable, FontError> {
    let (i, version) = be_u16(i)?;
    require_eq!(version, 0);
    
    let mut table = KernTable::default();
    let (mut i, n_tables) = be_u16(i)?;
    
    fn _cu_parser(i: &[u8]) -> nom::IResult<&[u8], (u16, u16, u8, u8)> {
        let (i, sw) = be_u16(i)?; 
        let (i, sx) = be_u16(i)?;
        let (i, sy) = be_u8(i)?;
        let (i, sz) = be_u8(i)?;

        Ok((i, (sw, sx, sy, sz)))
    }

    for _ in 0 .. n_tables {
        let (_version, length, format, coverage) = parse(&mut i, _cu_parser)?;
        debug!("format={}, coverage={:02x}", format, coverage);
        let data = parse(&mut i, |s| take(length as usize - 6).parse(s))?;
        match (format, coverage) {
            (0, 0x01) => parse_kern_format0(data, &mut table)?,
            (2, 0x01) => parse_kern_format2(data, &mut table)?,
            (f, _) => error!("invalid kern subtable format {}", f)
        }
    }
    Ok(table)
}
