use super::block::LinkedBlock;
use super::dir::Dir;
use super::file::File;
use super::{dirname, filename, realpath, FileIO, IO};

use crate::sys;
use crate::sys::ata::Drive;
use crate::sys::clk::{BootTime, EpochTime, RTC};
use crate::sys::console::Console;
use crate::sys::net::gw::NetGw;
use crate::sys::net::ip::NetIp;
use crate::sys::net::mac::NetMac;
use crate::sys::net::socket::tcp::TcpSocket;
use crate::sys::net::socket::udp::UdpSocket;
use crate::sys::net::usage::NetUsage;
use crate::sys::rng::Random;
use crate::sys::speaker::Speaker;
use crate::sys::vga::{VgaBuffer, VgaFont, VgaMode, VgaPalette};

use alloc::vec;
use alloc::vec::Vec;
use core::convert::TryFrom;
use core::convert::TryInto;

#[derive(PartialEq, Eq, Clone, Copy)]
#[repr(u8)]
pub enum DeviceType {
    Null = 0,
    File = 1,
    Console = 2,
    Random = 3,
    BootTime = 4,
    EpochTime = 5,
    RTC = 6,
    TcpSocket = 7,
    UdpSocket = 8,
    Drive = 9,
    VgaBuffer = 10,
    VgaFont = 11,
    VgaMode = 12,
    VgaPalette = 13,
    Speaker = 14,
    NetGw = 15,
    NetIp = 16,
    NetMac = 17,
    NetUsage = 18,
}

impl TryFrom<&[u8]> for DeviceType {
    type Error = ();

    fn try_from(buf: &[u8]) -> Result<Self, Self::Error> {
        match buf.first().ok_or(())? {
            0 => Ok(DeviceType::Null),
            1 => Ok(DeviceType::File),
            2 => Ok(DeviceType::Console),
            3 => Ok(DeviceType::Random),
            4 => Ok(DeviceType::BootTime),
            5 => Ok(DeviceType::EpochTime),
            6 => Ok(DeviceType::RTC),
            7 => Ok(DeviceType::TcpSocket),
            8 => Ok(DeviceType::UdpSocket),
            9 => Ok(DeviceType::Drive),
            10 => Ok(DeviceType::VgaBuffer),
            11 => Ok(DeviceType::VgaFont),
            12 => Ok(DeviceType::VgaMode),
            13 => Ok(DeviceType::VgaPalette),
            14 => Ok(DeviceType::Speaker),
            15 => Ok(DeviceType::NetGw),
            16 => Ok(DeviceType::NetIp),
            17 => Ok(DeviceType::NetMac),
            18 => Ok(DeviceType::NetUsage),
            _ => Err(()),
        }
    }
}

impl DeviceType {
    // Return a buffer for the file representing the device in the filesystem.
    // The first byte is the device type. The remaining bytes can be used to
    // store specific device informations.
    pub fn buf(self) -> Vec<u8> {
        let len = match self {
            DeviceType::RTC => RTC::size(),
            DeviceType::BootTime => BootTime::size(),
            DeviceType::EpochTime => EpochTime::size(),
            DeviceType::Console => Console::size(),
            DeviceType::TcpSocket => TcpSocket::size(),
            DeviceType::UdpSocket => UdpSocket::size(),
            DeviceType::Drive => Drive::size(),
            DeviceType::VgaBuffer => VgaBuffer::size(),
            DeviceType::VgaMode => VgaMode::size(),
            DeviceType::VgaPalette => VgaPalette::size(),
            DeviceType::NetGw => NetGw::size(),
            DeviceType::NetIp => NetIp::size(),
            DeviceType::NetMac => NetMac::size(),
            DeviceType::NetUsage => NetUsage::size(),
            _ => 1,
        };
        let mut res = vec![0; len];
        res[0] = self as u8; // Device type
        res
    }
}

#[derive(Debug, Clone)]
pub enum Device {
    Null,
    File(File),
    Console(Console),
    Random(Random),
    BootTime(BootTime),
    EpochTime(EpochTime),
    RTC(RTC),
    TcpSocket(TcpSocket),
    UdpSocket(UdpSocket),
    Drive(Drive),
    VgaBuffer(VgaBuffer),
    VgaFont(VgaFont),
    VgaMode(VgaMode),
    VgaPalette(VgaPalette),
    Speaker(Speaker),
    NetGw(NetGw),
    NetIp(NetIp),
    NetMac(NetMac),
    NetUsage(NetUsage),
}

impl TryFrom<&[u8]> for Device {
    type Error = ();

    fn try_from(buf: &[u8]) -> Result<Self, Self::Error> {
        match buf.try_into()? {
            DeviceType::Null => Ok(Device::Null),
            DeviceType::File => Ok(Device::File(File::new())),
            DeviceType::Console => Ok(Device::Console(Console::new())),
            DeviceType::Random => Ok(Device::Random(Random::new())),
            DeviceType::BootTime => Ok(Device::BootTime(BootTime::new())),
            DeviceType::EpochTime => Ok(Device::EpochTime(EpochTime::new())),
            DeviceType::RTC => Ok(Device::RTC(RTC::new())),
            DeviceType::TcpSocket => Ok(Device::TcpSocket(TcpSocket::new())),
            DeviceType::UdpSocket => Ok(Device::UdpSocket(UdpSocket::new())),
            DeviceType::VgaBuffer => Ok(Device::VgaBuffer(VgaBuffer::new())),
            DeviceType::VgaFont => Ok(Device::VgaFont(VgaFont::new())),
            DeviceType::VgaMode => Ok(Device::VgaMode(VgaMode::new())),
            DeviceType::VgaPalette => Ok(Device::VgaPalette(VgaPalette::new())),
            DeviceType::Speaker => Ok(Device::Speaker(Speaker::new())),
            DeviceType::NetGw => Ok(Device::NetGw(NetGw::new())),
            DeviceType::NetIp => Ok(Device::NetIp(NetIp::new())),
            DeviceType::NetMac => Ok(Device::NetMac(NetMac::new())),
            DeviceType::NetUsage => Ok(Device::NetUsage(NetUsage::new())),
            DeviceType::Drive if buf.len() > 2 => {
                let bus = buf[1];
                let dsk = buf[2];
                if let Some(drive) = Drive::open(bus, dsk) {
                    Ok(Device::Drive(drive))
                } else {
                    Err(())
                }
            }
            _ => Err(()),
        }
    }
}

impl Device {
    pub fn create(pathname: &str) -> Option<Self> {
        let pathname = realpath(pathname);
        let dirname = dirname(&pathname);
        let filename = filename(&pathname);
        if let Some(mut dir) = Dir::open(dirname) {
            if let Some(dir_entry) = dir.create_device(filename) {
                return Some(Device::File(dir_entry.into()));
            }
        }
        None
    }

    pub fn open(pathname: &str) -> Option<Self> {
        let pathname = realpath(pathname);
        let dirname = dirname(&pathname);
        let filename = filename(&pathname);
        if let Some(dir) = Dir::open(dirname) {
            if let Some(dir_entry) = dir.find(filename) {
                if dir_entry.is_device() {
                    let block = LinkedBlock::read(dir_entry.addr());
                    let data = block.data();
                    return data.try_into().ok();
                }
            }
        }
        None
    }

    pub fn size(&self) -> usize {
        match self {
            Device::Null => 0,
            Device::File(io) => io.size(),
            Device::Console(_) => Console::size(),
            Device::Random(_) => Random::size(),
            Device::BootTime(_) => BootTime::size(),
            Device::EpochTime(_) => EpochTime::size(),
            Device::RTC(_) => RTC::size(),
            Device::TcpSocket(_) => TcpSocket::size(),
            Device::UdpSocket(_) => UdpSocket::size(),
            Device::VgaBuffer(_) => sys::vga::buffer::Buffer::size(),
            Device::VgaFont(_) => VgaFont::size(),
            Device::VgaMode(_) => VgaMode::size(),
            Device::VgaPalette(_) => sys::vga::palette::Palette::size(),
            Device::Speaker(_) => Speaker::size(),
            Device::Drive(_) => Drive::size(),
            Device::NetGw(_) => NetGw::size(),
            Device::NetIp(_) => NetIp::size(),
            Device::NetMac(_) => NetMac::size(),
            Device::NetUsage(_) => NetUsage::size(),
        }
    }
}

impl FileIO for Device {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize, ()> {
        match self {
            Device::Null => Err(()),
            Device::File(io) => io.read(buf),
            Device::Console(io) => io.read(buf),
            Device::Random(io) => io.read(buf),
            Device::BootTime(io) => io.read(buf),
            Device::EpochTime(io) => io.read(buf),
            Device::RTC(io) => io.read(buf),
            Device::TcpSocket(io) => io.read(buf),
            Device::UdpSocket(io) => io.read(buf),
            Device::VgaBuffer(io) => io.read(buf),
            Device::VgaFont(io) => io.read(buf),
            Device::VgaMode(io) => io.read(buf),
            Device::VgaPalette(io) => io.read(buf),
            Device::Speaker(io) => io.read(buf),
            Device::Drive(io) => io.read(buf),
            Device::NetGw(io) => io.read(buf),
            Device::NetIp(io) => io.read(buf),
            Device::NetMac(io) => io.read(buf),
            Device::NetUsage(io) => io.read(buf),
        }
    }

    fn write(&mut self, buf: &[u8]) -> Result<usize, ()> {
        match self {
            Device::Null => Ok(0),
            Device::File(io) => io.write(buf),
            Device::Console(io) => io.write(buf),
            Device::Random(io) => io.write(buf),
            Device::BootTime(io) => io.write(buf),
            Device::EpochTime(io) => io.write(buf),
            Device::RTC(io) => io.write(buf),
            Device::TcpSocket(io) => io.write(buf),
            Device::UdpSocket(io) => io.write(buf),
            Device::VgaBuffer(io) => io.write(buf),
            Device::VgaFont(io) => io.write(buf),
            Device::VgaMode(io) => io.write(buf),
            Device::VgaPalette(io) => io.write(buf),
            Device::Speaker(io) => io.write(buf),
            Device::Drive(io) => io.write(buf),
            Device::NetGw(io) => io.write(buf),
            Device::NetIp(io) => io.write(buf),
            Device::NetMac(io) => io.write(buf),
            Device::NetUsage(io) => io.write(buf),
        }
    }

    fn close(&mut self) {
        match self {
            Device::Null => {}
            Device::File(io) => io.close(),
            Device::Console(io) => io.close(),
            Device::Random(io) => io.close(),
            Device::BootTime(io) => io.close(),
            Device::EpochTime(io) => io.close(),
            Device::RTC(io) => io.close(),
            Device::TcpSocket(io) => io.close(),
            Device::UdpSocket(io) => io.close(),
            Device::VgaBuffer(io) => io.close(),
            Device::VgaFont(io) => io.close(),
            Device::VgaMode(io) => io.close(),
            Device::VgaPalette(io) => io.close(),
            Device::Speaker(io) => io.close(),
            Device::Drive(io) => io.close(),
            Device::NetGw(io) => io.close(),
            Device::NetIp(io) => io.close(),
            Device::NetMac(io) => io.close(),
            Device::NetUsage(io) => io.close(),
        }
    }

    fn poll(&mut self, event: IO) -> bool {
        match self {
            Device::Null => false,
            Device::File(io) => io.poll(event),
            Device::Console(io) => io.poll(event),
            Device::Random(io) => io.poll(event),
            Device::BootTime(io) => io.poll(event),
            Device::EpochTime(io) => io.poll(event),
            Device::RTC(io) => io.poll(event),
            Device::TcpSocket(io) => io.poll(event),
            Device::UdpSocket(io) => io.poll(event),
            Device::VgaBuffer(io) => io.poll(event),
            Device::VgaFont(io) => io.poll(event),
            Device::VgaMode(io) => io.poll(event),
            Device::VgaPalette(io) => io.poll(event),
            Device::Speaker(io) => io.poll(event),
            Device::Drive(io) => io.poll(event),
            Device::NetGw(io) => io.poll(event),
            Device::NetIp(io) => io.poll(event),
            Device::NetMac(io) => io.poll(event),
            Device::NetUsage(io) => io.poll(event),
        }
    }
}
