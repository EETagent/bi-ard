#![no_std]
#![no_main]

use arduino_hal::eeprom::Eeprom;
use arduino_hal::prelude::*;

use panic_halt as _;
use ufmt::uwriteln;
use heapless::Vec;

const MAX_FILES: usize    = 8;
const MAX_NAME: usize     = 8;
const REC_SIZE: u16       = (2 + 1 + MAX_NAME) as u16;
const TABLE_START: u16    = 0;
const DATA_START: u16     = MAX_FILES as u16 * REC_SIZE;
const EEPROM_SIZE: u16    = 1024;
const EMPTY_ADDR: u16     = 0xFFFF;

#[derive(Copy, Clone, Debug)]
struct FileRec {
    addr: u16,
    len:  u8,
    name: [u8; MAX_NAME],
}

impl FileRec {
    fn load(eep: &mut Eeprom, i: usize) -> Self {
        let base = TABLE_START + (REC_SIZE * i as u16);
        let lo   = eep.read_byte(base) as u16;
        let hi   = eep.read_byte(base + 1) as u16;
        let addr = (hi << 8) | lo;
        let len  = eep.read_byte(base + 2);
        let mut name = [0; MAX_NAME];
        for j in 0..MAX_NAME {
            name[j] = eep.read_byte(base + 3 + j as u16);
        }
        FileRec { addr, len, name }
    }

    fn store(&self, eep: &mut Eeprom, i: usize) {
        let base = TABLE_START + (REC_SIZE * i as u16);
        let lo   = (self.addr & 0xFF) as u8;
        let hi   = (self.addr >> 8) as u8;
        eep.write_byte(base,     lo);
        eep.write_byte(base + 1, hi);
        eep.write_byte(base + 2, self.len);
        for j in 0..MAX_NAME {
            eep.write_byte(base + 3 + j as u16, self.name[j]);
        }
    }
}

fn cmd_format(eep: &mut Eeprom) {
    let empty = FileRec { addr: EMPTY_ADDR, len: 0, name: [0; MAX_NAME] };
    for i in 0..MAX_FILES {
        empty.store(eep, i);
    }
}

fn find_by_name(eep: &mut Eeprom, name: &[u8]) -> Option<usize> {
    for i in 0..MAX_FILES {
        let r = FileRec::load(eep, i);
        if r.addr != EMPTY_ADDR {
            let pos = r.name.iter().position(|&b| b == 0).unwrap_or(MAX_NAME);
            if pos == name.len() && &r.name[..pos] == name {
                return Some(i);
            }
        }
    }
    None
}

fn find_free_slot(eep: &mut Eeprom) -> Option<usize> {
    for i in 0..MAX_FILES {
        if FileRec::load(eep, i).addr == EMPTY_ADDR {
            return Some(i);
        }
    }
    None
}

fn find_data_place(eep: &mut Eeprom, len: u8) -> Option<u16> {
    let mut used: Vec<(u16,u16), MAX_FILES> = Vec::new();
    for i in 0..MAX_FILES {
        let r = FileRec::load(eep, i);
        if r.addr != EMPTY_ADDR {
            used.push((r.addr, r.addr + r.len as u16)).unwrap();
        }
    }
    for i in 0..used.len() {
        let mut min_idx = i;
        for j in (i + 1)..used.len() {
            if used[j].0 < used[min_idx].0 {
                min_idx = j;
            }
        }
        if i != min_idx {
            let temp = used[i];
            used[i] = used[min_idx];
            used[min_idx] = temp;
        }
    }

    // at front
    if used.is_empty() {
        if DATA_START + len as u16 <= EEPROM_SIZE {
            return Some(DATA_START);
        }
    } else {
        // before first
        if used[0].0 - DATA_START >= len as u16 {
            return Some(DATA_START);
        }
                // between
                for w in used.windows(2) {
                    let (_, end0) = w[0];
                    let (start1, _) = w[1];
                    if start1 - end0 >= len as u16 {
                        return Some(end0);
                    }
                }
        // after last
        let &(_, last_end) = used.last().unwrap();
        if last_end + len as u16 <= EEPROM_SIZE {
            return Some(last_end);
        }
    }
    None
}

fn cmd_remove(eep: &mut Eeprom, name: &[u8], serial: &mut impl ufmt::uWrite) {
    if let Some(idx) = find_by_name(eep, name) {
        let mut r = FileRec::load(eep, idx);
        r.addr = EMPTY_ADDR;
        r.len  = 0;
        r.name = [0; MAX_NAME];
        r.store(eep, idx);
        uwriteln!(serial, "OK remove").ok();
    } else {
        uwriteln!(serial, "ERR no such file").ok();
    }
}

fn cmd_read(eep: &mut Eeprom, name: &[u8], serial: &mut impl ufmt::uWrite) {
    if let Some(idx) = find_by_name(eep, name) {
        let r = FileRec::load(eep, idx);
        for off in 0..r.len {
            let b = eep.read_byte(r.addr + off as u16);
            ufmt::uwrite!(serial, "{}", b as char).ok();
        }
        uwriteln!(serial, "").ok();
    } else {
        uwriteln!(serial, "ERR no such file").ok();
    }
}

fn cmd_size(eep: &mut Eeprom, name: &[u8], serial: &mut impl ufmt::uWrite) {
    if let Some(idx) = find_by_name(eep, name) {
        let r = FileRec::load(eep, idx);
        uwriteln!(serial, "{}", r.len).ok();
    } else {
        uwriteln!(serial, "ERR no such file").ok();
    }
}

fn cmd_list(eep: &mut Eeprom, serial: &mut impl ufmt::uWrite) {
    for i in 0..MAX_FILES {
        let r = FileRec::load(eep, i);
        if r.addr != EMPTY_ADDR {
            let pos = r.name.iter().position(|&b| b==0).unwrap_or(MAX_NAME);
            for &c in &r.name[..pos] {
                ufmt::uwrite!(serial, "{}", c as char).ok();
            }
            uwriteln!(serial, " len={} addr={}", r.len, r.addr).ok();
        }
    }
}

fn cmd_defrag(eep: &mut Eeprom, serial: &mut impl ufmt::uWrite) {
    let mut tbl: Vec<FileRec, MAX_FILES> = Vec::new();
    for i in 0..MAX_FILES {
        let r = FileRec::load(eep, i);
        if r.addr != EMPTY_ADDR {
            tbl.push(r).unwrap()
        }
    }
    cmd_format(eep);
    let mut dp = DATA_START;
    for (i, mut r) in tbl.into_iter().enumerate() {
        for off in 0..r.len {
            let b = eep.read_byte(r.addr + off as u16);
            eep.write_byte(dp + off as u16, b);
        }
        r.addr = dp;
        r.store(eep, i);
        dp += r.len as u16;
    }
    uwriteln!(serial, "OK defrag").ok();
}

#[arduino_hal::entry]
fn main() -> ! {
    let dp     = arduino_hal::Peripherals::take().unwrap();
    let pins   = arduino_hal::pins!(dp);
    let mut serial = arduino_hal::default_serial!(dp, pins, 57600);

    let mut eep = Eeprom::new(dp.EEPROM);

    let mut buf = [0u8; 64];
    let mut len = 0;

    loop {
        if let Some(b) = nb::block!(serial.read()).ok() {
            match b {
                b'\r' | b'\n' => {
                    // parse!
                    let line = &buf[..len];
                    let mut parts = line.split(|&c| c == b' ');
                    if let Some(kw) = parts.next() {
                        match kw {
                            b"format" => {
                                cmd_format(&mut eep);
                                uwriteln!(serial, "OK format").ok();
                            }
                            b"list"   => cmd_list(&mut eep, &mut serial),
                            b"defrag" => cmd_defrag(&mut eep, &mut serial),
                            b"remove" => {
                                if let Some(n) = parts.next() {
                                    cmd_remove(&mut eep, n, &mut serial)
                                } else {
                                    uwriteln!(serial, "ERR syntax").ok();
                                }
                            }
                            b"read" => {
                                if let Some(n) = parts.next() {
                                    cmd_read(&mut eep, n, &mut serial)
                                } else {
                                    uwriteln!(serial, "ERR syntax").ok();
                                }
                            }
                            b"size" => {
                                if let Some(n) = parts.next() {
                                    cmd_size(&mut eep, n, &mut serial)
                                } else {
                                    uwriteln!(serial, "ERR syntax").ok();
                                }
                            }
                            b"create" => {
                                // create name len <data…>
                                if let (Some(n), Some(lb)) = (parts.next(), parts.next()) {
                                    if let Some(length) = core::str::from_utf8(lb)
                                        .ok()
                                        .and_then(|s| s.parse::<usize>().ok())
                                    {
                                        let mut data: Vec<u8, 64> = Vec::new();
                                        for chunk in parts {
                                            for &byte in chunk {
                                                if data.len() < length {
                                                    data.push(byte).unwrap();
                                                }
                                            }
                                        }
                                        if data.len() != length {
                                            uwriteln!(serial, "ERR data short").ok();
                                        } else if find_by_name(&mut eep, n).is_some() {
                                            uwriteln!(serial, "ERR name exists").ok();
                                        } else if let Some(slot) = find_free_slot(&mut eep) {
                                            if let Some(place) = find_data_place(&mut eep, length as u8) {
                                                for (i, &b) in data.iter().enumerate() {
                                                    eep.write_byte(place + i as u16, b);
                                                }
                                                let mut rec = FileRec {
                                                    addr: place,
                                                    len: length as u8,
                                                    name: [0; MAX_NAME],
                                                };
                                                let m = n.len().min(MAX_NAME);
                                                rec.name[..m].copy_from_slice(&n[..m]);
                                                rec.store(&mut eep, slot);
                                                uwriteln!(serial, "OK create").ok();
                                            } else {
                                                uwriteln!(serial, "ERR no data space").ok();
                                            }
                                        } else {
                                            uwriteln!(serial, "ERR no slot").ok();
                                        }
                                    } else {
                                        uwriteln!(serial, "ERR bad len").ok();
                                    }
                                } else {
                                    uwriteln!(serial, "ERR syntax").ok();
                                }
                            }
                            _ => {
                                uwriteln!(serial, "ERR unknown cmd").ok();
                            }
                        }
                    }
                    len = 0;
                }
                b if len < buf.len() => {
                    buf[len] = b;
                    len += 1;
                }
                _ => {}
            }
        }
    }
}
